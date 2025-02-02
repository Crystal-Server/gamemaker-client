enum LoginResult {
    OK = 0,
    NoAccount = 1,
    WrongPass = 2,
    NoAuth = 3,
    Unverified = 4,
    AlreadyIn = 5,
    GameBan = 6,
    GlobalBan = 7,
    Error = 8,
    MaxPlayers = 9,
}

enum RegisterResult {
    OK = 0,
    AccountExists = 1,
    UsedEmail = 2,
    InvalidEmail = 3,
    ShortPass = 4,
    InvalidName = 5,
    ShortName = 6,
    DiffPass = 7,
    Error = 8,
    LongName = 9,
    GlobalBan = 10,
    LongPass = 11,
    MaxAccounts = 12,
}

enum VariableQueueType {
    Null,
    Set,
    Remove,
}

enum SyncEvent {
    New = 0,
    Step = 1,
    End = 2,
    Once = 3,
}

enum CreateSync {
    Once = 0,
    Normal = 1,
}

enum P2PCode {
    AllGame = -1,
    CurrentSession = -2,
    CurrentRoom = -3,
    Server = -4,
}

function Player() constructor {
    id = -1;
    name = "";
    rm = "";
    syncs = [];
    variables = {};
}

function Sync() constructor {
    //index = -1; // Is this really necessary?
    kind = -1;
    sync_type = -1;
    event = -1;
    is_ending = false;
    variables = {};
}

function Administrator() constructor {
    can_ban = false;
    can_unban = false;
    can_kick = false;
}

function SyncIter() constructor {
	id = -1;
	name = "";
	sync_slot = -1;
	event = -1;
	kind = -1;
	variables = {};
}

global.__crystal_dll = {
    "_set_room": external_define("crystal_dll.dll", "__crystal_set_room", dll_cdecl, ty_real, 1, ty_string),

    "init": external_define("crystal_dll.dll", "__crystal_init", dll_cdecl, ty_real, 1, ty_string),
    "connect": external_define("crystal_dll.dll", "__crystal_connect", dll_cdecl, ty_real, 0),
    "update": external_define("crystal_dll.dll", "__crystal_update", dll_cdecl, ty_real, 0),
    "get_notification": external_define("crystal_dll.dll", "__crystal_get_notification", dll_cdecl, ty_string, 0),
    "is_connected": external_define("crystal_dll.dll", "__crystal_is_connected", dll_cdecl, ty_real, 0),
    "is_connecting": external_define("crystal_dll.dll", "__crystal_is_connecting", dll_cdecl, ty_real, 0),
    "is_loggedin": external_define("crystal_dll.dll", "__crystal_is_loggedin", dll_cdecl, ty_real, 0),
    "get_ping": external_define("crystal_dll.dll", "__crystal_get_ping", dll_cdecl, ty_real, 0),
    "set_game_token": external_define("crystal_dll.dll", "__crystal_set_game_token", dll_cdecl, ty_real, 1, ty_string),
    "disconnect": external_define("crystal_dll.dll", "__crystal_disconnect", dll_cdecl, ty_real, 0),
    "login": external_define("crystal_dll.dll", "__crystal_login", dll_cdecl, ty_real, 2, ty_string, ty_string),
    "login_with_token": external_define("crystal_dll.dll", "__crystal_login_with_token", dll_cdecl, ty_real, 2, ty_string, ty_string),
    "register": external_define("crystal_dll.dll", "__crystal_register", dll_cdecl, ty_real, 4, ty_string, ty_string, ty_string, ty_string),
    "get_player_id": external_define("crystal_dll.dll", "__crystal_get_player_id", dll_cdecl, ty_real, 0),
    "get_player_name": external_define("crystal_dll.dll", "__crystal_get_player_name", dll_cdecl, ty_string, 0),
    "set_variable": external_define("crystal_dll.dll", "__crystal_set_variable", dll_cdecl, ty_real, 2, ty_string, ty_string),
    "remove_variable": external_define("crystal_dll.dll", "__crystal_remove_variable", dll_cdecl, ty_real, 1, ty_string),
    "iter_other_players": external_define("crystal_dll.dll", "__crystal_iter_other_players", dll_cdecl, ty_string, 0),
    "other_player_count": external_define("crystal_dll.dll", "__crystal_other_player_count", dll_cdecl, ty_real, 0),
    "get_other_player": external_define("crystal_dll.dll", "__crystal_get_other_player", dll_cdecl, ty_string, 1, ty_real),
    "get_other_player_name": external_define("crystal_dll.dll", "__crystal_get_other_player_name", dll_cdecl, ty_string, 1, ty_string),
    "request_other_player_variable": external_define("crystal_dll.dll", "__crystal_request_other_player_variable", dll_cdecl, ty_real, 3, ty_real, ty_string, ty_real),
    "p2p": external_define("crystal_dll.dll", "__crystal_p2p", dll_cdecl, ty_real, 3, ty_real, ty_real, ty_string),
    "set_version": external_define("crystal_dll.dll", "__crystal_set_version", dll_cdecl, ty_real, 1, ty_real),
    "get_version": external_define("crystal_dll.dll", "__crystal_get_version", dll_cdecl, ty_real, 0),
	"get_server_version": external_define("crystal_dll.dll", "__crystal_get_server_version", dll_cdecl, ty_real, 0),
	"set_session": external_define("crystal_dll.dll", "__crystal_set_session", dll_cdecl, ty_real, 1, ty_string),
    "get_session": external_define("crystal_dll.dll", "__crystal_get_session", dll_cdecl, ty_string, 0),
    "get_open_playerini": external_define("crystal_dll.dll", "__crystal_get_open_playerini", dll_cdecl, ty_string, 0),
    "open_playerini": external_define("crystal_dll.dll", "__crystal_open_playerini", dll_cdecl, ty_real, 1, ty_string),
    "close_playerini": external_define("crystal_dll.dll", "__crystal_close_playerini", dll_cdecl, ty_real, 0),
    "has_playerini": external_define("crystal_dll.dll", "__crystal_has_playerini", dll_cdecl, ty_real, 2, ty_string, ty_string),
    "get_playerini": external_define("crystal_dll.dll", "__crystal_get_playerini", dll_cdecl, ty_string, 2, ty_string, ty_string),
    "set_playerini": external_define("crystal_dll.dll", "__crystal_set_playerini", dll_cdecl, ty_real, 3, ty_string, ty_string, ty_string),
    "remove_playerini": external_define("crystal_dll.dll", "__crystal_remove_playerini", dll_cdecl, ty_real, 2, ty_string, ty_string),
    "get_open_gameini": external_define("crystal_dll.dll", "__crystal_get_open_gameini", dll_cdecl, ty_string, 0),
    "open_gameini": external_define("crystal_dll.dll", "__crystal_open_gameini", dll_cdecl, ty_real, 1, ty_string),
    "close_gameini": external_define("crystal_dll.dll", "__crystal_close_gameini", dll_cdecl, ty_real, 0),
    "has_gameini": external_define("crystal_dll.dll", "__crystal_has_gameini", dll_cdecl, ty_real, 2, ty_string, ty_string),
    "get_gameini": external_define("crystal_dll.dll", "__crystal_get_gameini", dll_cdecl, ty_string, 2, ty_string, ty_string),
    "set_gameini": external_define("crystal_dll.dll", "__crystal_set_gameini", dll_cdecl, ty_real, 3, ty_string, ty_string, ty_string),
    "remove_gameini": external_define("crystal_dll.dll", "__crystal_remove_gameini", dll_cdecl, ty_real, 2, ty_string, ty_string),
    "has_achievement": external_define("crystal_dll.dll", "__crystal_has_achievement", dll_cdecl, ty_real, 1, ty_real),
    "get_achievement": external_define("crystal_dll.dll", "__crystal_get_achievement", dll_cdecl, ty_string, 1, ty_real),
    "has_reached_achievement": external_define("crystal_dll.dll", "__crystal_has_reached_achievement", dll_cdecl, ty_real, 1, ty_real),
    "get_reached_achievement": external_define("crystal_dll.dll", "__crystal_get_reached_achievement", dll_cdecl, ty_real, 1, ty_real),
    "reach_achievement": external_define("crystal_dll.dll", "__crystal_reach_achievement", dll_cdecl, ty_real, 1, ty_real),
    "has_highscore": external_define("crystal_dll.dll", "__crystal_has_highscore", dll_cdecl, ty_real, 1, ty_real),
    "get_highscore": external_define("crystal_dll.dll", "__crystal_get_highscore", dll_cdecl, ty_string, 1, ty_real),
    "has_score_highscore": external_define("crystal_dll.dll", "__crystal_has_score_highscore", dll_cdecl, ty_real, 1, ty_real),
    "get_score_highscore": external_define("crystal_dll.dll", "__crystal_get_score_highscore", dll_cdecl, ty_real, 1, ty_real),
    "set_score_highscore": external_define("crystal_dll.dll", "__crystal_set_score_highscore", dll_cdecl, ty_real, 2, ty_real, ty_real),
    "create_sync": external_define("crystal_dll.dll", "__crystal_create_sync", dll_cdecl, ty_real, 2, ty_real, ty_real),
    "destroy_sync": external_define("crystal_dll.dll", "__crystal_destroy_sync", dll_cdecl, ty_real, 1, ty_real),
    "set_variable_sync": external_define("crystal_dll.dll", "__crystal_set_variable_sync", dll_cdecl, ty_real, 3, ty_real, ty_string, ty_string),
    "remove_variable_sync": external_define("crystal_dll.dll", "__crystal_remove_variable_sync", dll_cdecl, ty_real, 2, ty_real, ty_string),
    "get_variable_other_sync": external_define("crystal_dll.dll", "__crystal_get_variable_other_sync", dll_cdecl, ty_real, 3, ty_real, ty_real, ty_string),
    "iter_other_syncs": external_define("crystal_dll.dll", "__crystal_iter_other_syncs", dll_cdecl, ty_string, 0),
    "is_player_admin": external_define("crystal_dll.dll", "__crystal_is_player_admin", dll_cdecl, ty_real, 1, ty_real),
    "get_player_admin": external_define("crystal_dll.dll", "__crystal_get_player_admin", dll_cdecl, ty_string, 1, ty_real),
    "player_kick": external_define("crystal_dll.dll", "__crystal_player_kick", dll_cdecl, ty_real, 2, ty_real, ty_string),
    "player_ban": external_define("crystal_dll.dll", "__crystal_player_ban", dll_cdecl, ty_real, 3, ty_real, ty_string, ty_real),
    "player_unban": external_define("crystal_dll.dll", "__crystal_player_unban", dll_cdecl, ty_real, 1, ty_real),
    "logout": external_define("crystal_dll.dll", "__crystal_logout", dll_cdecl, ty_real, 0),
    "request_other_sync_variable": external_define("crystal_dll.dll", "__crystal_request_other_sync_variable", dll_cdecl, ty_real, 4, ty_real, ty_real, ty_string, ty_real),
    "fetch_bdb": external_define("crystal_dll.dll", "__crystal_fetch_bdb", dll_cdecl, ty_real, 2, ty_string, ty_real),
    "set_bdb": external_define("crystal_dll.dll", "__crystal_set_bdb", dll_cdecl, ty_real, 2, ty_string, ty_string),
    "get_incoming_friends": external_define("crystal_dll.dll", "__crystal_get_incoming_friends", dll_cdecl, ty_string, 0),
    "get_outgoing_friends": external_define("crystal_dll.dll", "__crystal_get_outgoing_friends", dll_cdecl, ty_string, 0),
    "get_friends": external_define("crystal_dll.dll", "__crystal_get_friends", dll_cdecl, ty_string, 0),
    "send_outgoing_friend": external_define("crystal_dll.dll", "__crystal_send_outgoing_friend", dll_cdecl, ty_real, 1, ty_real),
    "remove_outgoing_friend": external_define("crystal_dll.dll", "__crystal_remove_outgoing_friend", dll_cdecl, ty_real, 1, ty_real),
    "deny_incoming_friend": external_define("crystal_dll.dll", "__crystal_deny_incoming_friend", dll_cdecl, ty_real, 1, ty_real),
    "accept_incoming_friend": external_define("crystal_dll.dll", "__crystal_accept_incoming_friend", dll_cdecl, ty_real, 1, ty_real),
    "remove_friend": external_define("crystal_dll.dll", "__crystal_remove_friend", dll_cdecl, ty_real, 1, ty_real),
    //"": external_define("crystal_dll.dll", "", dll_cdecl, ty_real, 0),
};

global.__crystal_callback_room = undefined;
global.__crystal_callback_p2p = undefined;
global.__crystal_callback_register = undefined;
global.__crystal_callback_login = undefined;
global.__crystal_callback_banned = undefined;
global.__crystal_callback_kicked = undefined;
global.__crystal_callback_disconnected = undefined;
global.__crystal_callback_login_token = undefined;
//global.__crystal_callback_data_update = undefined;
global.__crystal_callback_bdb = undefined;
global.__crystal_callback_update_variable = undefined;
global.__crystal_callback_update_sync_variable = undefined;

function crystal_set_callback_room(callback) {
    global.__crystal_callback_room = callback;
}

function crystal_set_callback_p2p(callback) {
    global.__crystal_callback_p2p = callback;
}

function crystal_set_callback_register(callback) {
    global.__crystal_callback_register = callback;
}

function crystal_set_callback_login(callback) {
    global.__crystal_callback_login = callback;
}

function crystal_set_callback_banned(callback) {
    global.__crystal_callback_banned = callback;
}

function crystal_set_callback_kicked(callback) {
    global.__crystal_callback_kicked = callback;
}

function crystal_set_callback_disconnected(callback) {
    global.__crystal_callback_disconnected = callback;
}

function crystal_set_callback_login_token(callback) {
    global.__crystal_callback_login_token = callback;
}

/*function crystal_set_callback_data_update(callback) {
    global.__crystal_callback_data_update = callback;
}*/

function crystal_set_callback_bdb(callback) {
    global.__crystal_callback_bdb = callback;
}

function crystal_set_callback_update_variable(callback) {
    global.__crystal_callback_update_variable = callback;
}

function crystal_set_callback_update_sync_variable(callback) {
    global.__crystal_callback_update_sync_variable = callback;
}

function crystal_init(game_id) {
    return external_call(global.__crystal_dll[$ "init"], game_id);
}

function crystal_connect() {
    return external_call(global.__crystal_dll[$ "connect"]);
}

function crystal_update() {
	var r = external_call(global.__crystal_dll[$ "update"]);
    var rm = string(room);
    if global.__crystal_callback_room != undefined
        rm = string(global.__crystal_callback_room());
    external_call(global.__crystal_dll[$ "_set_room"], rm);
    var notf = external_call(global.__crystal_dll[$ "get_notification"]);
    while string_length(notf) > 0 {
        var s = string_split(notf, ";");
		//show_debug_message(notf + "\t\t\t" + string(s));
        switch s[0] {
			case "login_token":
				if global.__crystal_callback_login_token != undefined
					global.__crystal_callback_login_token(base64_decode(s[1]));
				break;
            case "admin_action":
                switch s[1] {
                    case "1":
                        if global.__crystal_callback_banned != undefined
                            global.__crystal_callback_banned(base64_decode(s[2]), int64(s[3]));
                        break;
                    case "2":
                        if global.__crystal_callback_kicked != undefined
                            global.__crystal_callback_kicked(base64_decode(s[2]));
                        break;
                }
                break;
            case "banned":
                /*if global.__crystal_callback_banned != undefined
                    global.__crystal_callback_banned(base64_decode(s[1]), int64(s[2]));*/
                break;
            case "kicked":
                /*if global.__crystal_callback_kicked != undefined
                    global.__crystal_callback_kicked(base64_decode(s[1]));*/
                break;
            case "friend_status": // status->u64
                break;
            case "disconnected":
                if global.__crystal_callback_disconnected != undefined
                    global.__crystal_callback_disconnected();
                break;
            case "fetch_bdb":
                switch s[1] {
                    case "0":
                        if global.__crystal_callback_bdb != undefined
                            global.__crystal_callback_bdb(base64_decode(s[2]));
                        break;
                    case "1":
                        if global.__crystal_callback_bdb != undefined
                            global.__crystal_callback_bdb(base64_decode(s[2]), base64_decode(s[3]));
                        break;
                }
                break;
            case "login":
                if global.__crystal_callback_login != undefined
                    global.__crystal_callback_login(real(s[1]));
                break;
            case "login_ok":
                if global.__crystal_callback_login != undefined
                    global.__crystal_callback_login(LoginResult.OK);
				break;
			case "login_ban":
                if global.__crystal_callback_login != undefined
                    global.__crystal_callback_login(real(s[1]), base64_decode(s[2]), int64(s[3]));
                break;
            case "p2p":
                if global.__crystal_callback_p2p != undefined {
                    var _pid = -1;
                    if s[1] != "!"
                        _pid = real(s[1]);
                    global.__crystal_callback_p2p(_pid, real(s[2]), __decode_variable(s[3]));
                }
                break;
            case "register":
                if global.__crystal_callback_register != undefined
                    global.__crystal_callback_register(real(s[1]));
                break;
            case "player_logged_in": // pid->u64,name->string_base64,room->string_base64
                break;
            case "player_logged_out": // pid->u64
                break;
            case "reconnecting":
                break;
            case "server_message": // server_message->string_base64
                break;
            case "update_variable":
                if global.__crystal_callback_update_variable != undefined
                    global.__crystal_callback_update_variable(real(s[1]), base64_decode(s[2]), __decode_variable(s[3]), s[3] == "!!");
                break;
            case "update_sync_variable":
                if global.__crystal_callback_update_sync_variable != undefined
                    global.__crystal_callback_update_sync_variable(real(s[1]), real(s[2]), base64_decode(s[3]), __decode_variable(s[4]), s[4] == "!!");
                break;
            case "update_sync_removal": // pid->u64,slot->u64
                break;
            case "update_gameini": // file->string_base64,section->string_base64,key->string_base64,value->vari
                break;
            case "update_playerini": // file->string_base64,section->string_base64,key->string_base64,value->vari
                break;
            case "update_gameversion": // version->f64
                break;
            case "update_administrator": // pid->u64,admin->admin
                break;
            case "server_notification": // notif->string_base64
                break;
        }
        notf = external_call(global.__crystal_dll[$ "get_notification"]);
    }
    return r;
}

function crystal_is_connected() {
    return external_call(global.__crystal_dll[$ "is_connected"]);
}

function crystal_is_connecting() {
    return external_call(global.__crystal_dll[$ "is_connecting"]);
}

function crystal_is_loggedin() {
    return external_call(global.__crystal_dll[$ "is_loggedin"]);
}

function crystal_get_ping() {
    return external_call(global.__crystal_dll[$ "get_ping"]);
}

function crystal_set_game_token(token) {
    return external_call(global.__crystal_dll[$ "set_game_token"], token);
}

function crystal_disconnect() {
    return external_call(global.__crystal_dll[$ "disconnect"]);
}

function crystal_login(name, passw) {
    return external_call(global.__crystal_dll[$ "login"], name, passw);
}

function crystal_login_with_token(name, token) {
    return external_call(global.__crystal_dll[$ "login_with_token"], name, token);
}

function crystal_register(name, email, passw, repeat_passw) {
    return external_call(global.__crystal_dll[$ "register"]);
}

function crystal_get_player_id() {
    return external_call(global.__crystal_dll[$ "get_player_id"]);
}

function crystal_get_player_name() {
    return external_call(global.__crystal_dll[$ "get_player_name"]);
}

function crystal_set_variable(name, variable) {
    return external_call(global.__crystal_dll[$ "set_variable"], name, __encode_variable(variable));
}

function crystal_remove_variable(name) {
    return external_call(global.__crystal_dll[$ "remove_variable"], name);
}

function crystal_iter_other_players() {
	var ss = external_call(global.__crystal_dll[$ "iter_other_players"]);
	//show_debug_message(ss);
    var s = string_split(ss, ";");
	//show_debug_message(r + "\t\t\t" + string(s));
    var r = [];
	if array_length(s) == 1 && string_length(s[0]) == 0
		return r;
    for (var i = 0; i < array_length(s); i++)
        array_push(r, __decode_player(s[i]));
    return r;
}

function crystal_other_player_count() {
    return external_call(global.__crystal_dll[$ "other_player_count"]);
}

function crystal_get_other_player(pid) {
    return __decode_player(external_call(global.__crystal_dll[$ "get_other_player"], pid));
}

function crystal_get_other_player_name(name) {
    return __decode_player(external_call(global.__crystal_dll[$ "get_other_player_name"], name));
}

function crystal_request_other_player_variable(pid, name, request) {
    return external_call(global.__crystal_dll[$ "request_other_player_variable"], pid, name, request);
}

function crystal_p2p(target, mid, payload) {
    var s = string(array_length(payload));
    for (var i = 0; i < array_length(payload); i++)
        s += ";" + __encode_variable(payload[i]);
    return external_call(global.__crystal_dll[$ "p2p"], target, mid, s);
}

function crystal_set_version(version) {
    return external_call(global.__crystal_dll[$ "set_version"], version);
}

function crystal_get_version() {
    return external_call(global.__crystal_dll[$ "get_version"]);
}

function crystal_get_server_version() {
    return external_call(global.__crystal_dll[$ "get_server_version"]);
}

function crystal_set_session(session) {
    return external_call(global.__crystal_dll[$ "set_session"], session);
}

function crystal_get_session() {
    return external_call(global.__crystal_dll[$ "get_session"]);
}

function crystal_get_open_playerini() {
    return external_call(global.__crystal_dll[$ "get_open_playerini"]);
}

function crystal_open_playerini(file) {
    return external_call(global.__crystal_dll[$ "open_playerini"], file);
}

function crystal_close_playerini() {
    return external_call(global.__crystal_dll[$ "close_playerini"]);
}

function crystal_has_playerini(section, key) {
    return external_call(global.__crystal_dll[$ "has_playerini"], section, key);
}

function crystal_get_playerini(section, key) {
    return __decode_variable(external_call(global.__crystal_dll[$ "get_playerini"], section, key));
}

function crystal_set_playerini(section, key, value) {
    return external_call(global.__crystal_dll[$ "set_playerini"], section, key, __encode_variable(value));
}

function crystal_remove_playerini(section, key) {
    return external_call(global.__crystal_dll[$ "remove_playerini"], section, key);
}

function crystal_get_open_gameini() {
    return external_call(global.__crystal_dll[$ "get_open_gameini"]);
}

function crystal_open_gameini(file) {
    return external_call(global.__crystal_dll[$ "open_gameini"], file);
}

function crystal_close_gameini() {
    return external_call(global.__crystal_dll[$ "close_gameini"]);
}

function crystal_has_gameini(section, key) {
    return external_call(global.__crystal_dll[$ "has_gameini"], section, key);
}

function crystal_get_gameini(section, key) {
    return __decode_variable(external_call(global.__crystal_dll[$ "get_gameini"], section, key));
}

function crystal_set_gameini(section, key, value) {
    return external_call(global.__crystal_dll[$ "set_gameini"], section, key, __encode_variable(value));
}

function crystal_remove_gameini(section, key) {
    return external_call(global.__crystal_dll[$ "remove_gameini"], section, key);
}

function crystal_has_achievement(aid) {
    return external_call(global.__crystal_dll[$ "has_achievement"], aid);
}

function crystal_get_achievement(aid) {
    return external_call(global.__crystal_dll[$ "get_achievement"], aid);
}

function crystal_has_reached_achievement(aid) {
    return external_call(global.__crystal_dll[$ "has_reached_achievement"], aid);
}

function crystal_get_reached_achievement(aid) {
    return external_call(global.__crystal_dll[$ "get_reached_achievement"], aid);
}

function crystal_reach_achievement(aid) {
    return external_call(global.__crystal_dll[$ "reach_achievement"], aid);
}

function crystal_has_highscore(hid) {
    return external_call(global.__crystal_dll[$ "has_highscore"], hid);
}

function crystal_get_highscore(hid) {
    return external_call(global.__crystal_dll[$ "get_highscore"], hid);
}

function crystal_has_score_highscore(hid) {
    return external_call(global.__crystal_dll[$ "has_score_highscore"], hid);
}

function crystal_get_score_highscore(hid) {
    return external_call(global.__crystal_dll[$ "get_score_highscore"], hid);
}

function crystal_set_score_highscore(hid, score) {
    return external_call(global.__crystal_dll[$ "set_score_highscore"], hid, score);
}

function crystal_create_sync(sync_type, kind) {
    return external_call(global.__crystal_dll[$ "create_sync"], sync_type, kind);
}

function crystal_destroy_sync(sync) {
    return external_call(global.__crystal_dll[$ "destroy_sync"], sync);
}

function crystal_set_variable_sync(sync, name, value) {
    return external_call(global.__crystal_dll[$ "set_variable_sync"], sync, name, __encode_variable(value));
}

function crystal_remove_variable_sync(sync, name) {
    return external_call(global.__crystal_dll[$ "remove_variable_sync"], sync, name);
}

function crystal_get_variable_other_sync(pid, sync, name) {
    return external_call(global.__crystal_dll[$ "get_variable_other_sync"], pid, sync, name);
}

function crystal_iter_other_syncs() {
    var ss = external_call(global.__crystal_dll[$ "iter_other_syncs"]);
	//show_debug_message(ss);
    var s = string_split(ss, ";");
	//show_debug_message(ss + "\t\t\t" + string(s));
    var r = [];
	if array_length(s) == 1 && string_length(s[0]) == 0
		return r;
    for (var i = 0; i < array_length(s); i++)
        array_push(r, __decode_synciter(s[i]));
    return r;
}

function crystal_is_player_admin(pid) {
    return external_call(global.__crystal_dll[$ "is_player_admin"], pid);
}

function crystal_get_player_admin(pid) {
    return __decode_administrator(external_call(global.__crystal_dll[$ "get_player_admin"], pid));
}

function crystal_player_kick(pid, reason) {
    return external_call(global.__crystal_dll[$ "player_kick"], pid, reason);
}

function crystal_player_ban(pid, reason, unban_time) {
    return external_call(global.__crystal_dll[$ "player_ban"], pid, reason, unban_time);
}

function crystal_player_unban(pid) {
    return external_call(global.__crystal_dll[$ "player_unban"], pid);
}

function crystal_logout() {
    return external_call(global.__crystal_dll[$ "logout"]);
}

function crystal_request_other_sync_variable(pid, slot, name, request) {
    return external_call(global.__crystal_dll[$ "request_other_sync_variable"], pid, slot, name, request);
}

function crystal_fetch_bdb(name, request) {
    return external_call(global.__crystal_dll[$ "fetch_bdb"], name, request);
}

function crystal_set_bdb(name, data) {
    return external_call(global.__crystal_dll[$ "set_bdb"], name, buffer_base64_encode(data, 0, buffer_tell(data)));
}

function crystal_get_incoming_friends() {
    var s = string_split(external_call(global.__crystal_dll[$ "get_incoming_friends"]), ":");
    var r = [];
    var sz = real(s[0]);
    for (var i = 0; i < sz; i++)
        array_push(r, real(s[i + 1]));
    return r;
}

function crystal_get_outgoing_friends() {
    var s = string_split(external_call(global.__crystal_dll[$ "get_outgoing_friends"]), ":");
    var r = [];
    var sz = real(s[0]);
    for (var i = 0; i < sz; i++)
        array_push(r, real(s[i + 1]));
    return r;
}

function crystal_get_friends() {
    var s = string_split(external_call(global.__crystal_dll[$ "get_friends"]), ":");
    var r = [];
    var sz = real(s[0]);
    for (var i = 0; i < sz; i++)
        array_push(r, real(s[i + 1]));
    return r;
}

function crystal_send_outgoing_friend(pid) {
    return external_call(global.__crystal_dll[$ "send_outgoing_friend"], pid);
}

function crystal_remove_outgoing_friend(pid) {
    return external_call(global.__crystal_dll[$ "remove_outgoing_friend"], pid);
}

function crystal_deny_incoming_friend(pid) {
    return external_call(global.__crystal_dll[$ "deny_incoming_friend"], pid);
}

function crystal_accept_incoming_friend(pid) {
    return external_call(global.__crystal_dll[$ "accept_incoming_friend"], pid);
}

function crystal_remove_friend(pid) {
    return external_call(global.__crystal_dll[$ "remove_friend"], pid);
}

/*
function crystal_() {
    return external_call(global.__crystal_dll[$ ""]);
}
*/

function __decode_administrator(s) {
    s = string_split(s, ":");
    var a = new Administrator();
    a.can_ban = bool(real(s[0]));
    a.can_unban = bool(real(s[1]));
    a.can_kick = bool(real(s[2]));
    return a;

}

function __decode_player(s) {
	//show_debug_message(s);
    s = string_split(s, ":");
	//show_debug_message(s);
    var p = new Player();
	if s[0] == "!"
		return undefined;
    p.id = real(s[0]);
    p.name = base64_decode(s[1]);
    p.rm = base64_decode(s[2]);
    var ssize = real(s[3]);
    var vsize = real(s[4]);
    var o = 5;
    for (var i = 0; i < ssize; i++) {
        var sa = __decode_sync(base64_decode(s[o]));
        p.syncs[sa[0]] = sa[1];
        o++;
    }
    for (var i = 0; i < vsize; i++) {
        p.variables[$ base64_decode(s[o])] = __decode_variable(base64_decode(s[o + 1]));
        o += 2;
    }
    return p;
}

function __decode_sync(s) {
	//show_debug_message(s);
    s = string_split(s, ":");
	//show_debug_message(s);
    if s[1] == "!"
        return [real(s[0]), undefined];
    var sy = new Sync();
    sy.kind = real(s[1]);
    sy.sync_type = real(s[2]);
    sy.event = real(s[3]);
    sy.is_ending = bool(real(s[4]));
    var svari = real(real(s[5]));
	var o = 6;
    for (var i = 0; i < svari; i++) {
        sy.variables[$ base64_decode(s[o])] = __decode_variable(base64_decode(s[o + 1]));
		o += 2;
    }
    return [real(s[0]), sy];
}

function __encode_variable(vari) {
    switch typeof(vari) {
        case "undefined":
        case "null":
            return "!";
        case "int32":
        case "int64":
            return "0:" + string(vari);
        case "number":
            return "1:" + string(vari);
        case "bool":
            return "2:" + string(vari ? 1 : 0);
        case "string":
            return "3:" + base64_encode(vari);
        case "ref":
            if buffer_exists(vari)
                return buffer_base64_encode(vari, 0, buffer_tell(vari));
            show_error("Invalid variable type: " + typeof(vari) + " (" + string(vari) + ")", true);
        case "array":
            var s = "5:" + string(array_length(vari));
            for (var i = 0; i < array_length(vari); i++)
                s += ":" + base64_encode(__encode_variable(array_get(vari, i)));
            return s;
        case "struct":
            s = "6:" + string(variable_struct_names_count(vari));
            var v = variable_struct_get_names(vari);
            for (var i = 0; i < array_length(v); i++)
                s += ":" + base64_encode(__encode_variable(variable_struct_get(vari, v[i])));
            return s;
        default:
            show_error("Invalid variable type: " + typeof(vari), true);
    }
}

function __decode_variable(ss) {
	//show_debug_message(ss);
    var s = string_split(ss, ":");
    switch s[0] {
        case "!":
        case "!!":
            return undefined;
        case "0":
            return int64(s[1]);
        case "1":
            return real(s[1]);
        case "2":
            return bool(real(s[1]));
        case "3":
            return base64_decode(s[1]);
        case "4":
            return buffer_base64_decode(s[1]);
        case "5":
            var r = [];
            var sz = real(s[1]);
            for (var i = 0; i < sz; i++)
                array_push(r, __decode_variable(base64_decode(s[i + 2])));
            return r;
        case "6":
            r = {};
            sz = real(s[1]);
            for (var i = 0; i < sz; i += 2)
                r[$ base64_decode(s[i + 2])] = __decode_variable(base64_decode(s[i + 3]));
            return r;
    }
}

function __decode_synciter(s) {
	s = string_split(s, ":");
	var si = new SyncIter();
	si.id = real(s[0]);
	si.name = base64_decode(s[1]);
	si.sync_slot = real(s[2]);
	si.event = real(s[3]);
	si.kind = real(s[4]);
	var vsz = real(s[5]);
	for (var i = 6; i < array_length(s); i += 2)
		si.variables[$ base64_decode(s[i])] = __decode_variable(base64_decode(s[i + 1]));
	return si;
}
