use std::{
    collections::{HashMap, VecDeque},
    sync::LazyLock,
};

use base64::{Engine, prelude::BASE64_STANDARD};
use chrono::DateTime;
use crystal_server::{
    client::CrystalServer,
    types::{
        self, Achievement, AdminAction, Administrator, DataUpdate, Highscore, OptionalValue,
        Player, PlayerRequest, SyncIter, SyncType, Value,
    },
};
use futures_util::{StreamExt, pin_mut};
use gm_utils::gm_func;
use tokio::{runtime::Runtime, sync::Mutex};

static CRYSTAL: LazyLock<Mutex<CrystalServer>> =
    LazyLock::new(|| Mutex::new(CrystalServer::init("")));
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());
static NOTIFICATIONS: LazyLock<parking_lot::Mutex<VecDeque<String>>> =
    LazyLock::new(|| parking_lot::Mutex::new(VecDeque::default()));
static HAS_INIT: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static ROOM: LazyLock<parking_lot::RwLock<String>> =
    LazyLock::new(|| parking_lot::RwLock::new(String::new()));

#[gm_func]
pub fn __crystal_set_room(room: &str) {
    #[cfg(feature = "debug")]
    println!("_set_room({room:?})");
    RUNTIME.block_on(async {
        *ROOM.write() = room.to_string();
    });
}

#[gm_func]
pub fn __crystal_init(game_id: &str) {
    #[cfg(feature = "debug")]
    println!("init({game_id:?})");

    RUNTIME.block_on(async {
        let mut hinit = HAS_INIT.lock().await;
        if !*hinit {
            *hinit = true;
            drop(hinit);
            tracing_subscriber::fmt().init();
            let mut lock = CRYSTAL.lock().await;
            *lock = CrystalServer::init(game_id);
            lock.callback_set_room(Box::new(|| ROOM.read().clone()))
                .await;
            lock.callback_set_data_update(Box::new(|input| {
                RUNTIME.spawn(async {
                    NOTIFICATIONS.lock().push_back(match input {
                        DataUpdate::AdminAction(aa) => match aa {
                            AdminAction::Unban => String::from("admin_action;0"),
                            AdminAction::Ban(reason, unban_time) => {
                                format!(
                                    "admin_action;1;{};{unban_time}",
                                    BASE64_STANDARD.encode(reason)
                                )
                            }
                            AdminAction::Kick(reason) => {
                                format!("admin_action;2;{}", BASE64_STANDARD.encode(reason))
                            }
                        },
                        DataUpdate::Banned(reason, unban_time) => {
                            format!(
                                "banned;{};{}",
                                BASE64_STANDARD.encode(reason),
                                unban_time.timestamp()
                            )
                        }
                        DataUpdate::ChangeFriendStatus(status) => {
                            format!("friend_status;{status}")
                        }
                        DataUpdate::Disconnected() => String::from("disconnected"),
                        DataUpdate::FetchBdb(name, value) => {
                            if let Some(value) = value {
                                format!(
                                    "fetch_bdb;1;{};{}",
                                    BASE64_STANDARD.encode(name),
                                    BASE64_STANDARD.encode(value)
                                )
                            } else {
                                format!("fetch_bdb;0;{}", BASE64_STANDARD.encode(name))
                            }
                        }
                        DataUpdate::Kicked(reason) => {
                            format!("kicked;{}", BASE64_STANDARD.encode(reason))
                        }
                        DataUpdate::Login(code) => {
                            format!("login;{}", code as u64)
                        }
                        DataUpdate::LoginOk(pid, name) => {
                            format!("login_ok;{pid};{}", BASE64_STANDARD.encode(name))
                        }
                        DataUpdate::LoginBan(code, reason, unban_time) => {
                            format!(
                                "login_ban;{};{};{unban_time}",
                                code as u64,
                                BASE64_STANDARD.encode(reason)
                            )
                        }
                        DataUpdate::P2P(sender, mid, payload) => {
                            format!(
                                "p2p;{};{mid};{}",
                                if let Some(sender) = sender {
                                    sender.to_string()
                                } else {
                                    String::from("!")
                                },
                                encode_vari(&Value::Array(payload))
                            )
                        }
                        DataUpdate::Registration(code) => {
                            format!("register;{}", code as u64)
                        }
                        DataUpdate::PlayerLoggedIn(pid, name, room) => {
                            format!(
                                "player_logged_in;{pid};{};{}",
                                BASE64_STANDARD.encode(name),
                                BASE64_STANDARD.encode(room)
                            )
                        }
                        DataUpdate::PlayerLoggedOut(pid) => {
                            format!("player_logged_out;{pid}")
                        }
                        DataUpdate::Reconnecting() => String::from("reconnecting"),
                        DataUpdate::ServerMessage(message) => {
                            format!("server_message;{}", BASE64_STANDARD.encode(message))
                        }
                        DataUpdate::UpdateVariable(pid, name, value) => {
                            format!(
                                "update_variable;{pid};{};{}",
                                BASE64_STANDARD.encode(name),
                                if let OptionalValue::Some(value) = value {
                                    encode_vari(&value)
                                } else {
                                    String::from("!!")
                                }
                            )
                        }
                        DataUpdate::UpdateSyncVariable(pid, slot, name, value) => {
                            format!(
                                "update_sync_variable;{pid};{slot};{};{}",
                                BASE64_STANDARD.encode(name),
                                if let OptionalValue::Some(value) = value {
                                    encode_vari(&value)
                                } else {
                                    String::from("!!")
                                }
                            )
                        }
                        DataUpdate::UpdateSyncRemoval(pid, slot) => {
                            format!("update_sync_removal;{pid};{slot}")
                        }
                        DataUpdate::UpdateGameIni(file, section, key, value) => {
                            format!(
                                "update_gameini;{};{};{};{}",
                                if let Some(file) = file {
                                    BASE64_STANDARD.encode(file)
                                } else {
                                    String::from("!!")
                                },
                                BASE64_STANDARD.encode(section),
                                BASE64_STANDARD.encode(key),
                                if let OptionalValue::Some(value) = value {
                                    encode_vari(&value)
                                } else {
                                    String::from("!!")
                                }
                            )
                        }
                        DataUpdate::UpdatePlayerIni(file, section, key, value) => {
                            format!(
                                "update_playerini;{};{};{};{}",
                                if let Some(file) = file {
                                    BASE64_STANDARD.encode(file)
                                } else {
                                    String::from("!!")
                                },
                                BASE64_STANDARD.encode(section),
                                BASE64_STANDARD.encode(key),
                                if let OptionalValue::Some(value) = value {
                                    encode_vari(&value)
                                } else {
                                    String::from("!!")
                                }
                            )
                        }
                        DataUpdate::UpdateGameVersion(ver) => {
                            format!("update_gameversion;{ver}")
                        }
                        DataUpdate::UpdateAdministrator(pid, admin) => {
                            format!(
                                "update_administrator;{pid};{}",
                                if let Some(admin) = admin {
                                    format!(
                                        "{}:{}:{}",
                                        admin.can_ban, admin.can_unban, admin.can_kick
                                    )
                                } else {
                                    String::from("!")
                                }
                            )
                        }
                        DataUpdate::ServerNotification(notif) => {
                            format!("server_notification;{}", BASE64_STANDARD.encode(notif))
                        }
                        DataUpdate::LoginToken(token) => {
                            format!("login_token;{}", BASE64_STANDARD.encode(token))
                        }
                    });
                });
            }))
            .await;
        }
    });
}

#[gm_func]
pub fn __crystal_connect() {
    #[cfg(feature = "debug")]
    println!("connect()");
    // TODO: This should probably be async, not blocking (sync.)
    RUNTIME.spawn(async { CRYSTAL.lock().await.connect().await });
}

#[gm_func]
pub fn __crystal_update() -> bool {
    #[cfg(feature = "debug")]
    println!("update()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.update().await.is_ok() })
}

#[gm_func]
pub fn __crystal_get_notification() -> String {
    #[cfg(feature = "debug")]
    println!("get_notification()");
    NOTIFICATIONS.lock().pop_front().unwrap_or_default()
}

#[gm_func]
pub fn __crystal_is_connected() -> bool {
    #[cfg(feature = "debug")]
    println!("is_connected()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.is_connected().await })
}

#[gm_func]
pub fn __crystal_is_connecting() -> bool {
    #[cfg(feature = "debug")]
    println!("is_connecting()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.is_connecting().await })
}

#[gm_func]
pub fn __crystal_is_loggedin() -> bool {
    #[cfg(feature = "debug")]
    println!("is_loggedin()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.is_loggedin().await })
}

#[gm_func]
pub fn __crystal_get_ping() -> f64 {
    #[cfg(feature = "debug")]
    println!("get_ping()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.get_ping().await })
}

#[gm_func]
pub fn __crystal_set_game_token(token: &str) {
    #[cfg(feature = "debug")]
    println!("set_game_token({token:?})");
    RUNTIME.block_on(async {
        CRYSTAL.lock().await.set_game_token(token).await;
    })
}

#[gm_func]
pub fn __crystal_disconnect() {
    #[cfg(feature = "debug")]
    println!("disconnect()");
    RUNTIME.block_on(async {
        CRYSTAL.lock().await.disconnect().await;
    })
}

#[gm_func]
pub fn __crystal_login(name: &str, passw: &str) {
    #[cfg(feature = "debug")]
    println!("login({name:?}, {passw:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.login(name, passw).await;
    })
}

#[gm_func]
pub fn __crystal_login_with_token(name: &str, token: &str) {
    #[cfg(feature = "debug")]
    println!("login_with_token({name:?}, {token:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.login_with_token(name, token).await;
    })
}

#[gm_func]
pub fn __crystal_register(name: &str, email: &str, passw: &str, repeat_passw: &str) {
    #[cfg(feature = "debug")]
    println!("register({name:?}, {email:?}, {passw:?}, {repeat_passw:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .register(name, email, passw, repeat_passw)
            .await;
    })
}

#[gm_func]
pub fn __crystal_get_player_id() -> f64 {
    #[cfg(feature = "debug")]
    println!("get_player_id()");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .get_player_id()
            .await
            .map(|val| val as f64)
            .unwrap_or(-1.0)
    })
}

#[gm_func]
pub fn __crystal_get_player_name() -> String {
    #[cfg(feature = "debug")]
    println!("get_player_name()");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .get_player_name()
            .await
            .unwrap_or(String::new())
    })
}

#[gm_func]
pub fn __crystal_set_variable(name: &str, variable: &str) {
    #[cfg(feature = "debug")]
    println!("set_variable({name:?}, {variable:?})");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .set_variable(name, decode_vari(variable))
            .await;
    })
}

#[gm_func]
pub fn __crystal_remove_variable(name: &str) {
    #[cfg(feature = "debug")]
    println!("remove_variable({name:?})");
    RUNTIME.block_on(async {
        CRYSTAL.lock().await.remove_variable(name).await;
    })
}

#[gm_func]
pub fn __crystal_iter_other_players() -> String {
    #[cfg(feature = "debug")]
    println!("iter_other_players()");
    RUNTIME.block_on(async {
        let lock = CRYSTAL.lock().await;
        let iter = lock.iter_other_players().await;
        pin_mut!(iter);
        let mut res = String::new();
        while let Some((pid, player)) = iter.next().await {
            if !res.is_empty() {
                res.push(';');
            }
            res.push_str(&encode_player(pid, &player));
        }
        res
    })
}

#[gm_func]
pub fn __crystal_other_player_count() -> f64 {
    #[cfg(feature = "debug")]
    println!("other_player_count()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.other_player_count().await as f64 })
}

#[gm_func]
pub fn __crystal_get_other_player(pid: f64) -> String {
    #[cfg(feature = "debug")]
    println!("get_other_player({pid:?})");
    RUNTIME.block_on(async {
        if let Some(player) = CRYSTAL.lock().await.get_other_player(pid as u64).await {
            encode_player(pid as u64, &player)
        } else {
            String::from("!")
        }
    })
}

#[gm_func]
pub fn __crystal_get_other_player_name(name: &str) -> String {
    #[cfg(feature = "debug")]
    println!("get_other_player_name({name:?})");
    RUNTIME.block_on(async {
        if let Some((pid, player)) = CRYSTAL.lock().await.get_other_player_name(name).await {
            encode_player(pid, &player)
        } else {
            String::from("!")
        }
    })
}

#[gm_func]
pub fn __crystal_request_other_player_variable(pid: f64, name: &str, request: f64) {
    #[cfg(feature = "debug")]
    println!("request_other_player_variable({pid:?}, {name:?}, {request:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .request_other_player_variable(
                pid as u64,
                name,
                Some(Box::new(move |pid, name, vari| {
                    NOTIFICATIONS.lock().push_back(format!(
                        "player_variable_request;{request};{pid};{name};{}",
                        if let OptionalValue::Some(vari) = vari {
                            encode_vari(&vari)
                        } else {
                            String::from("!")
                        }
                    ));
                })),
            )
            .await;
    })
}

#[gm_func]
pub fn __crystal_p2p(target: f64, mid: f64, payload: &str) {
    #[cfg(feature = "debug")]
    println!("p2p({target:?}, {mid:?}, {payload:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .p2p(
                match target {
                    -1.0 => PlayerRequest::AllGame,
                    -2.0 => PlayerRequest::CurrentRoom,
                    -3.0 => PlayerRequest::CurrentSession,
                    -4.0 => PlayerRequest::Server,
                    _ => PlayerRequest::ID(target as u64),
                },
                mid as i16,
                {
                    let ps = payload.to_string();
                    let mut s = ps.split(";");
                    let mut data = Vec::new();
                    for _ in 0..s.next().unwrap().parse::<usize>().unwrap() {
                        data.push(decode_vari(s.next().unwrap()));
                    }
                    data
                },
            )
            .await;
    })
}

#[gm_func]
pub fn __crystal_set_version(version: f64) {
    #[cfg(feature = "debug")]
    println!("set_version({version:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.set_version(version).await;
    })
}

#[gm_func]
pub fn __crystal_get_version() -> f64 {
    #[cfg(feature = "debug")]
    println!("get_version()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.get_version().await })
}

#[gm_func]
pub fn __crystal_get_server_version() -> f64 {
    #[cfg(feature = "debug")]
    println!("get_server_version()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.get_server_version().await })
}

#[gm_func]
pub fn __crystal_set_session(session: &str) {
    #[cfg(feature = "debug")]
    println!("set_session({session:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.set_session(session).await;
    })
}

#[gm_func]
pub fn __crystal_get_session() -> String {
    #[cfg(feature = "debug")]
    println!("get_session()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.get_session().await })
}

#[gm_func]
pub fn __crystal_get_open_playerini() -> String {
    #[cfg(feature = "debug")]
    println!("get_open_playerini()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.get_open_playerini().await })
}

#[gm_func]
pub fn __crystal_open_playerini(file: &str) {
    #[cfg(feature = "debug")]
    println!("open_playerini({file:?})");
    RUNTIME.block_on(async {
        CRYSTAL.lock().await.open_playerini(file).await;
    })
}

#[gm_func]
pub fn __crystal_close_playerini() {
    #[cfg(feature = "debug")]
    println!("close_playerini()");
    RUNTIME.block_on(async {
        CRYSTAL.lock().await.close_playerini().await;
    })
}

#[gm_func]
pub fn __crystal_has_playerini(section: &str, key: &str) -> bool {
    #[cfg(feature = "debug")]
    println!("has_playerini({section:?}, {key:?})");
    RUNTIME.block_on(async { CRYSTAL.lock().await.has_playerini(section, key).await })
}

#[gm_func]
pub fn __crystal_get_playerini(section: &str, key: &str) -> String {
    #[cfg(feature = "debug")]
    println!("get_playerini({section:?}, {key:?})");
    RUNTIME.block_on(async {
        if let Some(vari) = CRYSTAL.lock().await.get_playerini(section, key).await {
            encode_vari(&vari)
        } else {
            String::from("!")
        }
    })
}

#[gm_func]
pub fn __crystal_set_playerini(section: &str, key: &str, vari: &str) {
    #[cfg(feature = "debug")]
    println!("set_playerini({section:?}, {key:?}, {vari:?})");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .set_playerini(section, key, decode_vari(vari))
            .await;
    })
}

#[gm_func]
pub fn __crystal_remove_playerini(section: &str, key: &str) {
    #[cfg(feature = "debug")]
    println!("remove_playerini({section:?}, {key:?})");
    RUNTIME.block_on(async {
        CRYSTAL.lock().await.remove_playerini(section, key).await;
    })
}

#[gm_func]
pub fn __crystal_get_open_gameini() -> String {
    #[cfg(feature = "debug")]
    println!("get_open_gameini()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.get_open_gameini().await })
}

#[gm_func]
pub fn __crystal_open_gameini(file: &str) {
    #[cfg(feature = "debug")]
    println!("open_gameini({file:?})");
    RUNTIME.block_on(async {
        CRYSTAL.lock().await.open_gameini(file).await;
    })
}

#[gm_func]
pub fn __crystal_close_gameini() {
    #[cfg(feature = "debug")]
    println!("close_gameini()");
    RUNTIME.block_on(async {
        CRYSTAL.lock().await.close_gameini().await;
    })
}

#[gm_func]
pub fn __crystal_has_gameini(section: &str, key: &str) -> bool {
    #[cfg(feature = "debug")]
    println!("has_gameini({section:?}, {key:?})");
    RUNTIME.block_on(async { CRYSTAL.lock().await.has_gameini(section, key).await })
}

#[gm_func]
pub fn __crystal_get_gameini(section: &str, key: &str) -> String {
    #[cfg(feature = "debug")]
    println!("get_gameini({section:?}, {key:?})");
    RUNTIME.block_on(async {
        if let Some(vari) = CRYSTAL.lock().await.get_gameini(section, key).await {
            encode_vari(&vari)
        } else {
            String::from("!")
        }
    })
}

#[gm_func]
pub fn __crystal_set_gameini(section: &str, key: &str, vari: &str) {
    #[cfg(feature = "debug")]
    println!("set_gameini({section:?}, {key:?}, {vari:?})");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .set_gameini(section, key, decode_vari(vari))
            .await;
    })
}

#[gm_func]
pub fn __crystal_remove_gameini(section: &str, key: &str) {
    #[cfg(feature = "debug")]
    println!("remove_gameini({section:?}, {key:?})");
    RUNTIME.block_on(async {
        CRYSTAL.lock().await.remove_gameini(section, key).await;
    })
}

#[gm_func]
pub fn __crystal_has_achievement(aid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("has_achievement({aid:?})");
    RUNTIME.block_on(async { CRYSTAL.lock().await.has_achievement(aid as u64).await })
}

#[gm_func]
pub fn __crystal_get_achievement(aid: f64) -> String {
    #[cfg(feature = "debug")]
    println!("get_achievement({aid:?})");
    RUNTIME.block_on(async {
        encode_achievement(&CRYSTAL.lock().await.get_achievement(aid as u64).await)
    })
}

#[gm_func]
pub fn __crystal_has_reached_achievement(aid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("has_reached_achievement({aid:?})");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .has_reached_achievement(aid as u64)
            .await
    })
}

#[gm_func]
pub fn __crystal_get_reached_achievement(aid: f64) -> f64 {
    #[cfg(feature = "debug")]
    println!("get_reached_achievement({aid:?})");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .get_reached_achievement(aid as u64)
            .await
            .map(|val| val as f64)
            .unwrap_or(f64::NAN)
    })
}

#[gm_func]
pub fn __crystal_reach_achievement(aid: f64) {
    #[cfg(feature = "debug")]
    println!("reach_achievement({aid:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.reach_achievement(aid as u64).await;
    })
}

#[gm_func]
pub fn __crystal_has_highscore(hid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("has_highscore({hid:?})");
    RUNTIME.block_on(async { CRYSTAL.lock().await.has_highscore(hid as u64).await })
}

#[gm_func]
pub fn __crystal_get_highscore(hid: f64) -> String {
    #[cfg(feature = "debug")]
    println!("get_highscore({hid:?})");
    RUNTIME
        .block_on(async { encode_highscore(&CRYSTAL.lock().await.get_highscore(hid as u64).await) })
}

#[gm_func]
pub fn __crystal_has_score_highscore(hid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("has_score_highscore({hid:?})");
    RUNTIME.block_on(async { CRYSTAL.lock().await.has_score_highscore(hid as u64).await })
}

#[gm_func]
pub fn __crystal_get_score_highscore(hid: f64) -> f64 {
    #[cfg(feature = "debug")]
    println!("get_score_highscore({hid:?})");

    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .get_score_highscore(hid as u64)
            .await
            .unwrap_or(f64::NAN)
    })
}

#[gm_func]
pub fn __crystal_set_score_highscore(hid: f64, score: f64) {
    #[cfg(feature = "debug")]
    println!("set_score_highscore({hid:?}, {score:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .set_score_highscore(hid as u64, score)
            .await;
    })
}

#[gm_func]
pub fn __crystal_create_sync(sync_type: f64, kind: f64) -> f64 {
    #[cfg(feature = "debug")]
    println!("create_sync({sync_type:?}, {kind:?})");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .create_sync(SyncType::try_from(sync_type as u8).unwrap(), kind as i16)
            .await as f64
    })
}

#[gm_func]
pub fn __crystal_destroy_sync(sync: f64) {
    #[cfg(feature = "debug")]
    println!("destroy_sync({sync:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.destroy_sync(sync as usize).await;
    })
}

#[gm_func]
pub fn __crystal_set_variable_sync(sync: f64, name: &str, value: &str) {
    #[cfg(feature = "debug")]
    println!("set_variable_sync({sync:?}, {name:?}, {value:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .set_variable_sync(sync as usize, name, decode_vari(value))
            .await;
    })
}

#[gm_func]
pub fn __crystal_remove_variable_sync(sync: f64, name: &str) {
    #[cfg(feature = "debug")]
    println!("remove_variable_sync({sync:?}, {name:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .remove_variable_sync(sync as usize, name)
            .await;
    })
}

#[gm_func]
pub fn __crystal_get_variable_other_sync(pid: f64, sync: f64, name: &str) -> String {
    #[cfg(feature = "debug")]
    println!("get_variable_other_sync({pid:?}, {sync:?}, {name:?})");
    RUNTIME.block_on(async {
        if let Some(vari) = CRYSTAL
            .lock()
            .await
            .get_variable_other_sync(pid as u64, sync as usize, name)
            .await
        {
            encode_vari(&vari)
        } else {
            String::from("!")
        }
    })
}

#[gm_func]
pub fn __crystal_iter_other_syncs() -> String {
    #[cfg(feature = "debug")]
    println!("iter_other_syncs()");
    RUNTIME.block_on(async {
        let lock = CRYSTAL.lock().await;
        let iter = lock.iter_other_syncs().await;
        pin_mut!(iter);
        let mut res = String::new();
        while let Some(sync) = iter.next().await {
            if !res.is_empty() {
                res.push(';');
            }
            res.push_str(&encode_synciter(&sync));
        }
        res
    })
}

#[gm_func]
pub fn __crystal_is_player_admin(pid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("is_player_admin({pid:?})");
    RUNTIME.block_on(async { CRYSTAL.lock().await.is_player_admin(pid as u64).await })
}

#[gm_func]
pub fn __crystal_get_player_admin(pid: f64) -> String {
    #[cfg(feature = "debug")]
    println!("get_player_admin({pid:?})");
    RUNTIME.block_on(async {
        if let Some(admin) = CRYSTAL.lock().await.get_player_admin(pid as u64).await {
            encode_administrator(&admin)
        } else {
            String::from("!")
        }
    })
}

#[gm_func]
pub fn __crystal_player_kick(pid: f64, reason: &str) -> bool {
    #[cfg(feature = "debug")]
    println!("player_kick({pid:?}, {reason:?})");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .player_kick(pid as u64, reason)
            .await
            .unwrap()
    })
}

#[gm_func]
pub fn __crystal_player_ban(pid: f64, reason: &str, unban_time: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("player_ban({pid:?}, {reason:?}, {unban_time:?})");
    RUNTIME.block_on(async {
        CRYSTAL
            .lock()
            .await
            .player_ban(
                pid as u64,
                reason,
                DateTime::from_timestamp(unban_time as i64, 0).unwrap(),
            )
            .await
            .unwrap()
    })
}

#[gm_func]
pub fn __crystal_player_unban(pid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("player_unban({pid:?})");
    RUNTIME.block_on(async { CRYSTAL.lock().await.player_unban(pid as u64).await.unwrap() })
}

#[gm_func]
pub fn __crystal_logout() -> bool {
    #[cfg(feature = "debug")]
    println!("logout()");
    RUNTIME.block_on(async { CRYSTAL.lock().await.logout().await.unwrap() })
}

#[gm_func]
pub fn __crystal_request_other_sync_variable(pid: f64, slot: f64, name: &str, request: f64) {
    #[cfg(feature = "debug")]
    println!("request_other_sync_variable({pid:?}, {slot:?}, {name:?}, {request:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .request_other_sync_variable(
                pid as u64,
                slot as usize,
                name,
                Some(Box::new(move |pid, name, vari| {
                    NOTIFICATIONS.lock().push_back(format!(
                        "sync_variable_request;{request};{pid};{name};{}",
                        if let OptionalValue::Some(vari) = vari {
                            encode_vari(&vari)
                        } else {
                            String::from("!")
                        }
                    ));
                })),
            )
            .await;
    })
}

#[gm_func]
pub fn __crystal_fetch_bdb(name: &str) {
    #[cfg(feature = "debug")]
    println!("fetch_bdb({name:?}, {request:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.fetch_bdb(name, None).await;
    })
}

#[gm_func]
pub fn __crystal_set_bdb(name: &str, data: &str) {
    #[cfg(feature = "debug")]
    println!("set_bdb({name:?}, {data:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .set_bdb(name, BASE64_STANDARD.decode(data).unwrap())
            .await;
    })
}

#[gm_func]
pub fn __crystal_get_incoming_friends() -> String {
    #[cfg(feature = "debug")]
    println!("get_incoming_friends()");
    RUNTIME.block_on(async {
        let fr = CRYSTAL
            .lock()
            .await
            .get_incoming_friends()
            .await
            .unwrap_or_default();
        let mut s = format!("{}", fr.len());
        for pid in fr {
            s.push_str(&format!(":{pid}"));
        }
        s
    })
}

#[gm_func]
pub fn __crystal_get_outgoing_friends() -> String {
    #[cfg(feature = "debug")]
    println!("get_outgoing_friends()");
    RUNTIME.block_on(async {
        let fr = CRYSTAL
            .lock()
            .await
            .get_outgoing_friends()
            .await
            .unwrap_or_default();
        let mut s = format!("{}", fr.len());
        for pid in fr {
            s.push_str(&format!(":{pid}"));
        }
        s
    })
}

#[gm_func]
pub fn __crystal_get_friends() -> String {
    #[cfg(feature = "debug")]
    println!("get_friends()");
    RUNTIME.block_on(async {
        let fr = CRYSTAL.lock().await.get_friends().await.unwrap_or_default();
        let mut s = format!("{}", fr.len());
        for pid in fr {
            s.push_str(&format!(":{pid}"));
        }
        s
    })
}

#[gm_func]
pub fn __crystal_send_outgoing_friend(pid: f64) {
    #[cfg(feature = "debug")]
    println!("send_outgoing_friend({pid:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.send_outgoing_friend(pid as u64).await;
    })
}

#[gm_func]
pub fn __crystal_remove_outgoing_friend(pid: f64) {
    #[cfg(feature = "debug")]
    println!("remove_outgoing_friend({pid:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .remove_outgoing_friend(pid as u64)
            .await;
    })
}

#[gm_func]
pub fn __crystal_deny_incoming_friend(pid: f64) {
    #[cfg(feature = "debug")]
    println!("deny_incoming_friend({pid:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.deny_incoming_friend(pid as u64).await;
    })
}

#[gm_func]
pub fn __crystal_accept_incoming_friend(pid: f64) {
    #[cfg(feature = "debug")]
    println!("accept_incoming_friend({pid:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL
            .lock()
            .await
            .accept_incoming_friend(pid as u64)
            .await;
    })
}

#[gm_func]
pub fn __crystal_remove_friend(pid: f64) {
    #[cfg(feature = "debug")]
    println!("remove_friend({pid:?})");
    RUNTIME.block_on(async {
        let _ = CRYSTAL.lock().await.remove_friend(pid as u64).await;
    })
}

fn encode_player(pid: u64, player: &Player) -> String {
    let mut s = format!(
        "{pid}:{}:{}:{}:{}",
        BASE64_STANDARD.encode(&player.name),
        BASE64_STANDARD.encode(&player.room),
        player.syncs.len(),
        player.variables.len()
    );
    for (index, sync) in player.syncs.iter().enumerate() {
        s.push_str(&format!(
            ":{}",
            BASE64_STANDARD.encode(encode_sync(index, sync))
        ));
    }
    for (name, value) in &player.variables {
        s.push_str(&format!(
            ":{}:{}",
            BASE64_STANDARD.encode(name),
            BASE64_STANDARD.encode(encode_vari(value))
        ));
    }
    s
}

fn encode_sync(index: usize, sync: &Option<types::Sync>) -> String {
    if let Some(sync) = sync {
        let mut s = format!(
            "{index}:{}:{}:{}:{}:{}",
            sync.kind,
            sync.sync_type as u64,
            sync.event as u64,
            sync.is_ending as u8,
            sync.variables.len()
        );
        for (name, value) in &sync.variables {
            s.push_str(&format!(
                ":{}:{}",
                BASE64_STANDARD.encode(name),
                BASE64_STANDARD.encode(encode_vari(value))
            ));
        }
        s
    } else {
        format!("{index}:!")
    }
}

fn encode_achievement(achievement: &Option<Achievement>) -> String {
    if let Some(achi) = achievement {
        format!(
            "{}:{}:{}",
            BASE64_STANDARD.encode(&achi.name),
            BASE64_STANDARD.encode(&achi.description),
            if let Some(unlocked) = achi.unlocked {
                unlocked.to_string()
            } else {
                String::from("!")
            }
        )
    } else {
        String::from("!")
    }
}

fn encode_highscore(highscore: &Option<Highscore>) -> String {
    if let Some(hscr) = highscore {
        let mut s = format!(
            "{}:{}",
            BASE64_STANDARD.encode(&hscr.name),
            hscr.scores.len()
        );
        for (pid, score) in hscr.scores.iter() {
            s.push_str(&format!(":{}:{score}", **pid));
        }
        s
    } else {
        String::from("!")
    }
}

fn encode_synciter(siter: &SyncIter) -> String {
    let mut s = format!(
        "{}:{}:{}:{}:{}:{}",
        siter.player_id,
        BASE64_STANDARD.encode(&siter.player_name),
        siter.slot,
        siter.event as u64,
        siter.kind,
        siter.variables.len()
    );
    for (name, value) in &siter.variables {
        s.push_str(&format!(
            ":{}:{}",
            BASE64_STANDARD.encode(name),
            BASE64_STANDARD.encode(encode_vari(value)),
        ));
    }
    s
}

fn encode_administrator(admin: &Administrator) -> String {
    format!("{}:{}:{}", admin.can_ban, admin.can_unban, admin.can_kick)
}

fn encode_vari(vari: &Value) -> String {
    match vari {
        Value::Null => String::from("!"),
        Value::Int(val) => format!("0:{val}"),
        Value::Float(val) => format!("1:{val}"),
        Value::Bool(val) => format!("2:{}", *val as u8),
        Value::String(val) => format!("3:{}", BASE64_STANDARD.encode(val)),
        Value::Buffer(val) => format!("4:{}", BASE64_STANDARD.encode(val)),
        Value::Array(val) => {
            let mut s = format!("5:{}", val.len());
            for val in val {
                s.push_str(&format!(":{}", BASE64_STANDARD.encode(encode_vari(val))));
            }
            s
        }
        Value::Struct(val) => {
            let mut s = format!("6:{}", val.len());
            for (name, val) in val {
                s.push_str(&format!(
                    ":{}:{}",
                    BASE64_STANDARD.encode(name),
                    BASE64_STANDARD.encode(encode_vari(val))
                ));
            }
            s
        }
    }
}

fn decode_vari(s: &str) -> Value {
    let mut s = s.split(":");
    match s.next() {
        Some("!") => Value::Null,
        Some("0") => Value::Int(s.next().unwrap().parse::<i64>().unwrap()),
        Some("1") => Value::Float(s.next().unwrap().parse::<f64>().unwrap()),
        Some("2") => Value::Bool(s.next().unwrap().parse::<i64>().unwrap() != 0),
        Some("3") => Value::String(
            String::from_utf8_lossy(&BASE64_STANDARD.decode(s.next().unwrap()).unwrap())
                .to_string(),
        ),
        Some("4") => Value::Buffer(BASE64_STANDARD.decode(s.next().unwrap()).unwrap()),
        Some("5") => Value::Array({
            let mut v = Vec::new();
            for _ in 0..s.next().unwrap().parse::<usize>().unwrap() {
                v.push(decode_vari(&String::from_utf8_lossy(
                    &BASE64_STANDARD.decode(s.next().unwrap()).unwrap(),
                )));
            }
            v
        }),
        Some("6") => Value::Struct({
            let mut v = HashMap::new();
            for _ in 0..s.next().unwrap().parse::<usize>().unwrap() {
                let name =
                    String::from_utf8_lossy(&BASE64_STANDARD.decode(s.next().unwrap()).unwrap())
                        .to_string();
                v.insert(
                    name,
                    decode_vari(&String::from_utf8_lossy(
                        &BASE64_STANDARD.decode(s.next().unwrap()).unwrap(),
                    )),
                );
            }
            v
        }),
        Some(_) => Value::Null,
        None => Value::Null,
    }
}
