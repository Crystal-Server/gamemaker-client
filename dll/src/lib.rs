use std::{
    cell::LazyCell,
    collections::{HashMap, VecDeque},
    ffi::CString,
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
use parking_lot::Mutex;
use tokio::runtime::Runtime;

static mut CRYSTAL: LazyCell<Mutex<CrystalServer>> =
    LazyCell::new(|| Mutex::new(CrystalServer::init("")));
static mut RUNTIME: LazyCell<Runtime> = LazyCell::new(|| Runtime::new().unwrap());
static mut NOTIFICATIONS: LazyCell<Mutex<VecDeque<String>>> =
    LazyCell::new(|| Mutex::new(VecDeque::default()));
static mut HAS_INIT: LazyCell<Mutex<bool>> = LazyCell::new(|| Mutex::new(false));

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn init(game_id: *mut i8) {
    let mut hinit = HAS_INIT.lock();
    if !*hinit {
        *hinit = true;
        let mut lock = CRYSTAL.lock();
        *lock = CrystalServer::init(CString::from_raw(game_id).to_str().unwrap());
        RUNTIME.block_on(async {
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
                        format!("reg;{}", code as u64)
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
                                String::from("!")
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
                                String::from("!")
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
                                String::from("!")
                            },
                            BASE64_STANDARD.encode(section),
                            BASE64_STANDARD.encode(key),
                            if let OptionalVariable::Some(value) = value {
                                encode_vari(&value)
                            } else {
                                String::from("!")
                            }
                        )
                    }
                    DataUpdate::UpdatePlayerIni(file, section, key, value) => {
                        format!(
                            "update_playerini;{};{};{};{}",
                            if let Some(file) = file {
                                BASE64_STANDARD.encode(file)
                            } else {
                                String::from("!")
                            },
                            BASE64_STANDARD.encode(section),
                            BASE64_STANDARD.encode(key),
                            if let OptionalVariable::Some(value) = value {
                                encode_vari(&value)
                            } else {
                                String::from("!")
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
                });
            }))
            .await;
        })
    }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn connect() {
    let mut lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.connect().await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn update() -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.update().await.is_ok() })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_notification() -> *const i8 {
    let mut ret = CString::default();
    if let Some(notif) = NOTIFICATIONS.lock().pop_front() {
        ret = CString::new(notif).unwrap();
    }
    ret.as_ptr()
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn is_connected() -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.is_connected().await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn is_connecting() -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.is_connecting().await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn is_loggedin() -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.is_loggedin().await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_ping() -> f64 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_ping().await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn set_game_token(token: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.set_game_token(CString::from_raw(token).to_str().unwrap())
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn disconnect() {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.disconnect().await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn login(name: *mut i8, passw: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .login(
                CString::from_raw(name).to_str().unwrap(),
                CString::from_raw(passw).to_str().unwrap(),
            )
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn login_with_token(name: *mut i8, token: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .login_with_token(
                CString::from_raw(name).to_str().unwrap(),
                CString::from_raw(token).to_str().unwrap(),
            )
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn register(
    name: *mut i8,
    email: *mut i8,
    passw: *mut i8,
    repeat_passw: *mut i8,
) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .register(
                CString::from_raw(name).to_str().unwrap(),
                CString::from_raw(email).to_str().unwrap(),
                CString::from_raw(passw).to_str().unwrap(),
                CString::from_raw(repeat_passw).to_str().unwrap(),
            )
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_player_id() -> f64 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.get_player_id()
            .await
            .map(|val| val as f64)
            .unwrap_or(-1.0)
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_player_name() -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let name = CString::new(lock.get_player_name().await.unwrap_or(String::new())).unwrap();
        name.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn set_variable(name: *mut i8, variable: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.set_variable(
            CString::from_raw(name).to_str().unwrap(),
            decode_vari(CString::from_raw(variable).to_str().unwrap()),
        )
        .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn remove_variable(name: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.remove_variable(CString::from_raw(name).to_str().unwrap())
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn iter_other_players() -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let iter = lock.iter_other_players().await;
        pin_mut!(iter);
        let mut res = String::new();
        while let Some((pid, player)) = iter.next().await {
            res.push_str(&format!("{};", encode_player(pid, &player)));
        }
        let res = CString::new(res).unwrap();
        res.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn other_players_count() -> f64 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.other_players_count().await as f64 })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_other_player(pid: f64) -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(
            if let Some(player) = lock.get_other_player(pid as u64).await {
                encode_player(pid as u64, &player)
            } else {
                String::from("!")
            },
        )
        .unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_other_player_name(name: *mut i8) -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(
            if let Some((pid, player)) = lock
                .get_other_player_name(CString::from_raw(name).to_str().unwrap())
                .await
            {
                encode_player(pid, &player)
            } else {
                String::from("!")
            },
        )
        .unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn request_other_player_variable(pid: f64, name: *mut i8, request: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .request_other_player_variable(
                pid as u64,
                CString::from_raw(name).to_str().unwrap(),
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

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn p2p(target: f64, mid: f64, payload: *mut i8) {
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
                    let cs = CString::from_raw(payload);
                    let cst = cs.to_string_lossy();
                    let mut s = cst.split(";");
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

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn set_version(version: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.set_version(version).await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_version() -> f64 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_version().await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_server_version() -> f64 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.get_server_version().await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_session() -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(lock.get_session().await).unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_open_playerini() -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(lock.get_open_playerini().await).unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn open_playerini(file: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.open_playerini(CString::from_raw(file).to_str().unwrap())
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn close_playerini() {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.close_playerini().await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn has_playerini(section: *mut i8, key: *mut i8) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.has_playerini(
            CString::from_raw(section).to_str().unwrap(),
            CString::from_raw(key).to_str().unwrap(),
        )
        .await
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_playerini(section: *mut i8, key: *mut i8) -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(
            if let Some(vari) = lock
                .get_playerini(
                    CString::from_raw(section).to_str().unwrap(),
                    CString::from_raw(key).to_str().unwrap(),
                )
                .await
            {
                encode_vari(&vari)
            } else {
                String::from("!")
            },
        )
        .unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn set_playerini(section: *mut i8, key: *mut i8, vari: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.set_playerini(
            CString::from_raw(section).to_str().unwrap(),
            CString::from_raw(key).to_str().unwrap(),
            decode_vari(CString::from_raw(vari).to_str().unwrap()),
        )
        .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn remove_playerini(section: *mut i8, key: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.remove_playerini(
            CString::from_raw(section).to_str().unwrap(),
            CString::from_raw(key).to_str().unwrap(),
        )
        .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_open_gameini() -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(lock.get_open_gameini().await).unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn open_gameini(file: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.open_gameini(CString::from_raw(file).to_str().unwrap())
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn close_gameini() {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.close_gameini().await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn has_gameini(section: *mut i8, key: *mut i8) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.has_gameini(
            CString::from_raw(section).to_str().unwrap(),
            CString::from_raw(key).to_str().unwrap(),
        )
        .await
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_gameini(section: *mut i8, key: *mut i8) -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(
            if let Some(vari) = lock
                .get_gameini(
                    CString::from_raw(section).to_str().unwrap(),
                    CString::from_raw(key).to_str().unwrap(),
                )
                .await
            {
                encode_vari(&vari)
            } else {
                String::from("!")
            },
        )
        .unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn set_gameini(section: *mut i8, key: *mut i8, vari: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.set_gameini(
            CString::from_raw(section).to_str().unwrap(),
            CString::from_raw(key).to_str().unwrap(),
            decode_vari(CString::from_raw(vari).to_str().unwrap()),
        )
        .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn remove_gameini(section: *mut i8, key: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.remove_gameini(
            CString::from_raw(section).to_str().unwrap(),
            CString::from_raw(key).to_str().unwrap(),
        )
        .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn has_achievement(aid: f64) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_achievement(aid as u64).await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_achievement(aid: f64) -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(encode_achievement(&lock.get_achievement(aid as u64).await)).unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn has_reached_achievement(aid: f64) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_reached_achievement(aid as u64).await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_reached_achievement(aid: f64) -> f64 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.get_reached_achievement(aid as u64)
            .await
            .map(|val| val as f64)
            .unwrap_or(f64::NAN)
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn reach_achievement(aid: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.reach_achievement(aid as u64).await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn has_highscore(aid: f64) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_highscore(aid as u64).await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_highscore(hid: f64) -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(encode_highscore(&lock.get_highscore(hid as u64).await)).unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn has_score_highscore(hid: f64) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.has_score_highscore(hid as u64).await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_score_highscore(hid: f64) -> f64 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.get_score_highscore(hid as u64)
            .await
            .unwrap_or(f64::NAN)
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn set_score_highscore(hid: f64, score: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.set_score_highscore(hid as u64, score).await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn create_sync(sync_type: f64, kind: f64) -> f64 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.create_sync(SyncType::try_from(sync_type as u8).unwrap(), kind as i16)
            .await as f64
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn destroy_sync(sync: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.destroy_sync(sync as usize).await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn set_variable_sync(sync: f64, name: *mut i8, value: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .set_variable_sync(
                sync as usize,
                CString::from_raw(name).to_str().unwrap(),
                decode_vari(CString::from_raw(value).to_str().unwrap()),
            )
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn remove_variable_sync(sync: f64, name: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .remove_variable_sync(sync as usize, CString::from_raw(name).to_str().unwrap())
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_variable_other_sync(
    pid: f64,
    sync: f64,
    name: *mut i8,
) -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(
            if let Some(vari) = lock
                .get_variable_other_sync(
                    pid as u64,
                    sync as usize,
                    CString::from_raw(name).to_str().unwrap(),
                )
                .await
            {
                encode_vari(&vari)
            } else {
                String::from("!")
            },
        )
        .unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn iter_other_syncs() -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let iter = lock.iter_other_syncs().await;
        pin_mut!(iter);
        let mut res = String::new();
        while let Some(sync) = iter.next().await {
            res.push_str(&format!("1:{};", encode_synciter(&sync)));
        }
        res.push('0');
        let res = CString::new(res).unwrap();
        res.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn is_player_admin(pid: f64) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.is_player_admin(pid as u64).await })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_player_admin(pid: f64) -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new(
            if let Some(admin) = lock.get_player_admin(pid as u64).await {
                encode_administrator(&admin)
            } else {
                String::from("!")
            },
        )
        .unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn player_kick(pid: f64, reason: *mut i8) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.player_kick(pid as u64, CString::from_raw(reason).to_str().unwrap())
            .await
            .unwrap()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn player_ban(pid: f64, reason: *mut i8, unban_time: f64) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        lock.player_ban(
            pid as u64,
            CString::from_raw(reason).to_str().unwrap(),
            DateTime::from_timestamp(unban_time as i64, 0).unwrap(),
        )
        .await
        .unwrap()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn player_unban(pid: f64) -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.player_unban(pid as u64).await.unwrap() })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn logout() -> bool {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async { lock.logout().await.unwrap() })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn request_other_sync_variable(
    pid: f64,
    slot: f64,
    name: *mut i8,
    request: f64,
) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .request_other_sync_variable(
                pid as u64,
                slot as usize,
                CString::from_raw(name).to_str().unwrap(),
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

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn fetch_bdb(name: *mut i8, request: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .fetch_bdb(
                CString::from_raw(name).to_str().unwrap(),
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

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn set_bdb(name: *mut i8, data: *mut i8) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock
            .set_bdb(
                CString::from_raw(name).to_str().unwrap(),
                BASE64_STANDARD
                    .decode(CString::from_raw(data).to_str().unwrap())
                    .unwrap(),
            )
            .await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_incoming_friends() -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new({
            let fr = lock.get_incoming_friends().await;
            let mut s = format!("{}", fr.len());
            for pid in fr {
                s.push_str(&format!(":{pid}"));
            }
            s
        })
        .unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_outgoing_friends() -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new({
            let fr = lock.get_outgoing_friends().await;
            let mut s = format!("{}", fr.len());
            for pid in fr {
                s.push_str(&format!(":{pid}"));
            }
            s
        })
        .unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn get_friends() -> *const i8 {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let s = CString::new({
            let fr = lock.get_friends().await;
            let mut s = format!("{}", fr.len());
            for pid in fr {
                s.push_str(&format!(":{pid}"));
            }
            s
        })
        .unwrap();
        s.as_ptr()
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn send_outgoing_friend(pid: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.send_outgoing_friend(pid as u64).await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn remove_outgoing_friend(pid: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.remove_outgoing_friend(pid as u64).await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn deny_incoming_friend(pid: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.deny_incoming_friend(pid as u64).await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn accept_incoming_friend(pid: f64) {
    let lock = CRYSTAL.lock();
    RUNTIME.block_on(async {
        let _ = lock.accept_incoming_friend(pid as u64).await;
    })
}

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn remove_friend(pid: f64) {
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
            encode_vari(value)
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
