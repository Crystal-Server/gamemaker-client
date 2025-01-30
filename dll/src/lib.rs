use std::{
    cell::LazyCell,
    collections::{HashMap, VecDeque},
    ffi::CString,
};

use base64::{prelude::BASE64_STANDARD, Engine};
use crystal_server::core::{
    client::CrystalServer,
    types::{self, AdminAction, DataUpdate, OptionalVariable, Player, PlayerRequest, Variable},
};
use futures_util::{pin_mut, StreamExt};
use parking_lot::Mutex;
use tokio::runtime::Runtime;

static mut CRYSTAL: LazyCell<Mutex<CrystalServer>> =
    LazyCell::new(|| Mutex::new(CrystalServer::init("GAME ID"))); // This must be set at compile-time to correctly connect.
static mut RUNTIME: LazyCell<Runtime> = LazyCell::new(|| Runtime::new().unwrap());
static mut NOTIFICATIONS: LazyCell<Mutex<VecDeque<String>>> =
    LazyCell::new(|| Mutex::new(VecDeque::default()));

#[no_mangle]
#[allow(clippy::missing_safety_doc, static_mut_refs)]
pub unsafe extern "cdecl" fn init() {
    let lock = CRYSTAL.lock();
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
            res.push_str(&format!("1:{};", encode_player(pid, &player)));
        }
        res.push('0');
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

fn encode_player(pid: u64, player: &Player) -> String {
    let mut s = format!(
        "{pid}:{}:{}:{}:{}",
        BASE64_STANDARD.encode(&player.name),
        BASE64_STANDARD.encode(&player.room),
        player.syncs.len(),
        player.variables.len()
    );
    for (index, sync) in player.syncs.iter().enumerate() {
        s.push_str(&format!(":{}", encode_sync(index, sync)));
    }
    for (name, value) in &player.variables {
        s.push_str(&format!(
            ":{}:{}",
            BASE64_STANDARD.encode(name),
            encode_vari(value)
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
                encode_vari(value)
            ));
        }
        s
    } else {
        format!("{index}:!")
    }
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
                s.push_str(&BASE64_STANDARD.encode(format!(":{}", encode_vari(val))));
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
