use std::{
    collections::{HashMap, VecDeque},
    sync::LazyLock,
};

use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::DateTime;
use crystal_server::core::{
    client::CrystalServer,
    types::{
        self, Achievement, AdminAction, Administrator, DataUpdate, Highscore, OptionalVariable,
        Player, PlayerRequest, SyncIter, SyncType, Variable,
    },
};
use futures_util::{pin_mut, StreamExt};
use gm_utils::gm_func;
use parking_lot::Mutex;
use tokio::runtime::Runtime;

static CRYSTAL: LazyLock<Mutex<CrystalServer>> =
    LazyLock::new(|| Mutex::new(CrystalServer::init("")));
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());
static NOTIFICATIONS: LazyLock<Mutex<VecDeque<String>>> =
    LazyLock::new(|| Mutex::new(VecDeque::default()));
static HAS_INIT: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));
static ROOM: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

#[gm_func]
pub fn __crystal_set_room(room: &str) {
    #[cfg(feature = "debug")]
    println!("_set_room({room:?})");
    *ROOM.lock() = room.to_string();
}

#[gm_func]
pub fn __crystal_init(game_id: &str) {
    #[cfg(feature = "debug")]
    println!("init({game_id:?})");
    let mut hinit = HAS_INIT.lock();
    if !*hinit {
        *hinit = true;
        tracing_subscriber::fmt().init();
        let mut lock = CRYSTAL.lock();
        *lock = CrystalServer::init(game_id);
        RUNTIME.block_on(async {
            lock.callback_set_room(Box::new(|| ROOM.lock().clone()))
                .await;
            lock.callback_set_data_update(Box::new(|input| {
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
                            encode_vari(&Variable::Array(payload))
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
                            if let OptionalVariable::Some(value) = value {
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
                            if let OptionalVariable::Some(value) = value {
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
                            if let OptionalVariable::Some(value) = value {
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
                            if let OptionalVariable::Some(value) = value {
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
                                format!("{}:{}:{}", admin.can_ban, admin.can_unban, admin.can_kick)
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
            }))
            .await;
        })
    }
}

#[gm_func]
pub fn __crystal_connect() {
    #[cfg(feature = "debug")]
    println!("connect()");
    let mut lock = CRYSTAL.lock();
    // TODO: This should probably be async, not blocking (sync.)
    RUNTIME.block_on(async { lock.connect().await });
}

#[gm_func]
pub fn __crystal_update() -> bool {
    #[cfg(feature = "debug")]
    println!("update()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.update().await.is_ok() })
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.is_connected().await })
}

#[gm_func]
pub fn __crystal_is_connecting() -> bool {
    #[cfg(feature = "debug")]
    println!("is_connecting()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.is_connecting().await })
}

#[gm_func]
pub fn __crystal_is_loggedin() -> bool {
    #[cfg(feature = "debug")]
    println!("is_loggedin()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.is_loggedin().await })
}

#[gm_func]
pub fn __crystal_get_ping() -> f64 {
    #[cfg(feature = "debug")]
    println!("get_ping()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_ping().await })
}

#[gm_func]
pub fn __crystal_set_game_token(token: &str) {
    #[cfg(feature = "debug")]
    println!("set_game_token({token:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.set_game_token(token).await;
    })
}

#[gm_func]
pub fn __crystal_disconnect() {
    #[cfg(feature = "debug")]
    println!("disconnect()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.disconnect().await;
    })
}

#[gm_func]
pub fn __crystal_login(name: &str, passw: &str) {
    #[cfg(feature = "debug")]
    println!("login({name:?}, {passw:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.login(name, passw).await;
    })
}

#[gm_func]
pub fn __crystal_login_with_token(name: &str, token: &str) {
    #[cfg(feature = "debug")]
    println!("login_with_token({name:?}, {token:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.login_with_token(name, token).await;
    })
}

#[gm_func]
pub fn __crystal_register(name: &str, email: &str, passw: &str, repeat_passw: &str) {
    #[cfg(feature = "debug")]
    println!("register({name:?}, {email:?}, {passw:?}, {repeat_passw:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.register(name, email, passw, repeat_passw).await;
    })
}

#[gm_func]
pub fn __crystal_get_player_id() -> f64 {
    #[cfg(feature = "debug")]
    println!("get_player_id()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.get_player_id()
            .await
            .map(|val| val as f64)
            .unwrap_or(-1.0)
    })
}

#[gm_func]
pub fn __crystal_get_player_name() -> String {
    #[cfg(feature = "debug")]
    println!("get_player_name()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_player_name().await.unwrap_or(String::new()) })
}

#[gm_func]
pub fn __crystal_set_variable(name: &str, variable: &str) {
    #[cfg(feature = "debug")]
    println!("set_variable({name:?}, {variable:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.set_variable(name, decode_vari(variable)).await;
    })
}

#[gm_func]
pub fn __crystal_remove_variable(name: &str) {
    #[cfg(feature = "debug")]
    println!("remove_variable({name:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.remove_variable(name).await;
    })
}

#[gm_func]
pub fn __crystal_iter_other_players() -> String {
    #[cfg(feature = "debug")]
    println!("iter_other_players()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.other_player_count().await as f64 })
}

#[gm_func]
pub fn __crystal_get_other_player(pid: f64) -> String {
    #[cfg(feature = "debug")]
    println!("get_other_player({pid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        if let Some(player) = lock.get_other_player(pid as u64).await {
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        if let Some((pid, player)) = lock.get_other_player_name(name).await {
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .request_other_player_variable(
                pid as u64,
                name,
                Some(Box::new(move |pid, name, vari| {
                    NOTIFICATIONS.lock().push_back(format!(
                        "player_variable_request;{request};{pid};{name};{}",
                        if let OptionalVariable::Some(vari) = vari {
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.set_version(version).await;
    })
}

#[gm_func]
pub fn __crystal_get_version() -> f64 {
    #[cfg(feature = "debug")]
    println!("get_version()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_version().await })
}

#[gm_func]
pub fn __crystal_get_server_version() -> f64 {
    #[cfg(feature = "debug")]
    println!("get_server_version()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_server_version().await })
}

#[gm_func]
pub fn __crystal_set_session(session: &str) {
    #[cfg(feature = "debug")]
    println!("set_session({session:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.set_session(session).await;
    })
}

#[gm_func]
pub fn __crystal_get_session() -> String {
    #[cfg(feature = "debug")]
    println!("get_session()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_session().await })
}

#[gm_func]
pub fn __crystal_get_open_playerini() -> String {
    #[cfg(feature = "debug")]
    println!("get_open_playerini()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_open_playerini().await })
}

#[gm_func]
pub fn __crystal_open_playerini(file: &str) {
    #[cfg(feature = "debug")]
    println!("open_playerini({file:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.open_playerini(file).await;
    })
}

#[gm_func]
pub fn __crystal_close_playerini() {
    #[cfg(feature = "debug")]
    println!("close_playerini()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.close_playerini().await;
    })
}

#[gm_func]
pub fn __crystal_has_playerini(section: &str, key: &str) -> bool {
    #[cfg(feature = "debug")]
    println!("has_playerini({section:?}, {key:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_playerini(section, key).await })
}

#[gm_func]
pub fn __crystal_get_playerini(section: &str, key: &str) -> String {
    #[cfg(feature = "debug")]
    println!("get_playerini({section:?}, {key:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        if let Some(vari) = lock.get_playerini(section, key).await {
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.set_playerini(section, key, decode_vari(vari)).await;
    })
}

#[gm_func]
pub fn __crystal_remove_playerini(section: &str, key: &str) {
    #[cfg(feature = "debug")]
    println!("remove_playerini({section:?}, {key:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.remove_playerini(section, key).await;
    })
}

#[gm_func]
pub fn __crystal_get_open_gameini() -> String {
    #[cfg(feature = "debug")]
    println!("get_open_gameini()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_open_gameini().await })
}

#[gm_func]
pub fn __crystal_open_gameini(file: &str) {
    #[cfg(feature = "debug")]
    println!("open_gameini({file:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.open_gameini(file).await;
    })
}

#[gm_func]
pub fn __crystal_close_gameini() {
    #[cfg(feature = "debug")]
    println!("close_gameini()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.close_gameini().await;
    })
}

#[gm_func]
pub fn __crystal_has_gameini(section: &str, key: &str) -> bool {
    #[cfg(feature = "debug")]
    println!("has_gameini({section:?}, {key:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_gameini(section, key).await })
}

#[gm_func]
pub fn __crystal_get_gameini(section: &str, key: &str) -> String {
    #[cfg(feature = "debug")]
    println!("get_gameini({section:?}, {key:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        if let Some(vari) = lock.get_gameini(section, key).await {
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.set_gameini(section, key, decode_vari(vari)).await;
    })
}

#[gm_func]
pub fn __crystal_remove_gameini(section: &str, key: &str) {
    #[cfg(feature = "debug")]
    println!("remove_gameini({section:?}, {key:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.remove_gameini(section, key).await;
    })
}

#[gm_func]
pub fn __crystal_has_achievement(aid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("has_achievement({aid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_achievement(aid as u64).await })
}

#[gm_func]
pub fn __crystal_get_achievement(aid: f64) -> String {
    #[cfg(feature = "debug")]
    println!("get_achievement({aid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { encode_achievement(&lock.get_achievement(aid as u64).await) })
}

#[gm_func]
pub fn __crystal_has_reached_achievement(aid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("has_reached_achievement({aid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_reached_achievement(aid as u64).await })
}

#[gm_func]
pub fn __crystal_get_reached_achievement(aid: f64) -> f64 {
    #[cfg(feature = "debug")]
    println!("get_reached_achievement({aid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.get_reached_achievement(aid as u64)
            .await
            .map(|val| val as f64)
            .unwrap_or(f64::NAN)
    })
}

#[gm_func]
pub fn __crystal_reach_achievement(aid: f64) {
    #[cfg(feature = "debug")]
    println!("reach_achievement({aid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.reach_achievement(aid as u64).await;
    })
}

#[gm_func]
pub fn __crystal_has_highscore(hid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("has_highscore({hid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_highscore(hid as u64).await })
}

#[gm_func]
pub fn __crystal_get_highscore(hid: f64) -> String {
    #[cfg(feature = "debug")]
    println!("get_highscore({hid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { encode_highscore(&lock.get_highscore(hid as u64).await) })
}

#[gm_func]
pub fn __crystal_has_score_highscore(hid: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("has_score_highscore({hid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_score_highscore(hid as u64).await })
}

#[gm_func]
pub fn __crystal_get_score_highscore(hid: f64) -> f64 {
    #[cfg(feature = "debug")]
    println!("get_score_highscore({hid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.get_score_highscore(hid as u64)
            .await
            .unwrap_or(f64::NAN)
    })
}

#[gm_func]
pub fn __crystal_set_score_highscore(hid: f64, score: f64) {
    #[cfg(feature = "debug")]
    println!("set_score_highscore({hid:?}, {score:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.set_score_highscore(hid as u64, score).await;
    })
}

#[gm_func]
pub fn __crystal_create_sync(sync_type: f64, kind: f64) -> f64 {
    #[cfg(feature = "debug")]
    println!("create_sync({sync_type:?}, {kind:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.create_sync(SyncType::try_from(sync_type as u8).unwrap(), kind as i16)
            .await as f64
    })
}

#[gm_func]
pub fn __crystal_destroy_sync(sync: f64) {
    #[cfg(feature = "debug")]
    println!("destroy_sync({sync:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.destroy_sync(sync as usize).await;
    })
}

#[gm_func]
pub fn __crystal_set_variable_sync(sync: f64, name: &str, value: &str) {
    #[cfg(feature = "debug")]
    println!("set_variable_sync({sync:?}, {name:?}, {value:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .set_variable_sync(sync as usize, name, decode_vari(value))
            .await;
    })
}

#[gm_func]
pub fn __crystal_remove_variable_sync(sync: f64, name: &str) {
    #[cfg(feature = "debug")]
    println!("remove_variable_sync({sync:?}, {name:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.remove_variable_sync(sync as usize, name).await;
    })
}

#[gm_func]
pub fn __crystal_get_variable_other_sync(pid: f64, sync: f64, name: &str) -> String {
    #[cfg(feature = "debug")]
    println!("get_variable_other_sync({pid:?}, {sync:?}, {name:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        if let Some(vari) = lock
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.is_player_admin(pid as u64).await })
}

#[gm_func]
pub fn __crystal_get_player_admin(pid: f64) -> String {
    #[cfg(feature = "debug")]
    println!("get_player_admin({pid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        if let Some(admin) = lock.get_player_admin(pid as u64).await {
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.player_kick(pid as u64, reason).await.unwrap() })
}

#[gm_func]
pub fn __crystal_player_ban(pid: f64, reason: &str, unban_time: f64) -> bool {
    #[cfg(feature = "debug")]
    println!("player_ban({pid:?}, {reason:?}, {unban_time:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.player_ban(
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.player_unban(pid as u64).await.unwrap() })
}

#[gm_func]
pub fn __crystal_logout() -> bool {
    #[cfg(feature = "debug")]
    println!("logout()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.logout().await.unwrap() })
}

#[gm_func]
pub fn __crystal_request_other_sync_variable(pid: f64, slot: f64, name: &str, request: f64) {
    #[cfg(feature = "debug")]
    println!("request_other_sync_variable({pid:?}, {slot:?}, {name:?}, {request:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .request_other_sync_variable(
                pid as u64,
                slot as usize,
                name,
                Some(Box::new(move |pid, name, vari| {
                    NOTIFICATIONS.lock().push_back(format!(
                        "sync_variable_request;{request};{pid};{name};{}",
                        if let OptionalVariable::Some(vari) = vari {
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
pub fn __crystal_fetch_bdb(name: &str, request: f64) {
    #[cfg(feature = "debug")]
    println!("fetch_bdb({name:?}, {request:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .fetch_bdb(
                name,
                Some(Box::new(move |pid, name, vari| {
                    NOTIFICATIONS.lock().push_back(format!(
                        "bdb_request;{request};{pid};{name};{}",
                        if let OptionalVariable::Some(vari) = vari {
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
pub fn __crystal_set_bdb(name: &str, data: &str) {
    #[cfg(feature = "debug")]
    println!("set_bdb({name:?}, {data:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .set_bdb(name, BASE64_STANDARD.decode(data).unwrap())
            .await;
    })
}

#[gm_func]
pub fn __crystal_get_incoming_friends() -> String {
    #[cfg(feature = "debug")]
    println!("get_incoming_friends()");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let fr = lock.get_incoming_friends().await;
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let fr = lock.get_outgoing_friends().await;
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let fr = lock.get_friends().await;
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
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.send_outgoing_friend(pid as u64).await;
    })
}

#[gm_func]
pub fn __crystal_remove_outgoing_friend(pid: f64) {
    #[cfg(feature = "debug")]
    println!("remove_outgoing_friend({pid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.remove_outgoing_friend(pid as u64).await;
    })
}

#[gm_func]
pub fn __crystal_deny_incoming_friend(pid: f64) {
    #[cfg(feature = "debug")]
    println!("deny_incoming_friend({pid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.deny_incoming_friend(pid as u64).await;
    })
}

#[gm_func]
pub fn __crystal_accept_incoming_friend(pid: f64) {
    #[cfg(feature = "debug")]
    println!("accept_incoming_friend({pid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.accept_incoming_friend(pid as u64).await;
    })
}

#[gm_func]
pub fn __crystal_remove_friend(pid: f64) {
    #[cfg(feature = "debug")]
    println!("remove_friend({pid:?})");
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.remove_friend(pid as u64).await;
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

fn encode_vari(vari: &Variable) -> String {
    match vari {
        Variable::Null => String::from("!"),
        Variable::Int(val) => format!("0:{val}"),
        Variable::Float(val) => format!("1:{val}"),
        Variable::Bool(val) => format!("2:{}", *val as u8),
        Variable::String(val) => format!("3:{}", BASE64_STANDARD.encode(val)),
        Variable::ByteArray(val) => format!("4:{}", BASE64_STANDARD.encode(val)),
        Variable::Array(val) => {
            let mut s = format!("5:{}", val.len());
            for val in val {
                s.push_str(&format!(":{}", BASE64_STANDARD.encode(encode_vari(val))));
            }
            s
        }
        Variable::Struct(val) => {
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

fn decode_vari(s: &str) -> Variable {
    let mut s = s.split(":");
    match s.next() {
        Some("!") => Variable::Null,
        Some("0") => Variable::Int(s.next().unwrap().parse::<i64>().unwrap()),
        Some("1") => Variable::Float(s.next().unwrap().parse::<f64>().unwrap()),
        Some("2") => Variable::Bool(s.next().unwrap().parse::<i64>().unwrap() != 0),
        Some("3") => Variable::String(
            String::from_utf8_lossy(&BASE64_STANDARD.decode(s.next().unwrap()).unwrap())
                .to_string(),
        ),
        Some("4") => Variable::ByteArray(BASE64_STANDARD.decode(s.next().unwrap()).unwrap()),
        Some("5") => Variable::Array({
            let mut v = Vec::new();
            for _ in 0..s.next().unwrap().parse::<usize>().unwrap() {
                v.push(decode_vari(&String::from_utf8_lossy(
                    &BASE64_STANDARD.decode(s.next().unwrap()).unwrap(),
                )));
            }
            v
        }),
        Some("6") => Variable::Struct({
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
        Some(_) => Variable::Null,
        None => Variable::Null,
    }
}
