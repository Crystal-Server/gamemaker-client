/// @feather ignore GM1041

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

function CrystalPlayer() constructor {
    id = -1;
    name = "";
    rm = "";
    syncs = [];
    variables = {};
}

function CrystalSync() constructor {
    //index = -1; // Is this really necessary?
    kind = -1;
    sync_type = -1;
    event = -1;
    is_ending = false;
    variables = {};
}

function CrystalAdministrator() constructor {
    can_ban = false;
    can_unban = false;
    can_kick = false;
}

function CrystalSyncIter() constructor {
	id = -1;
	name = "";
	sync_slot = -1;
	event = -1;
	kind = -1;
	variables = {};
}

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
    return __crystal_init(game_id);
}

function crystal_connect() {
    return __crystal_connect();
}

function crystal_update() {
	var r = __crystal_update();
    var rm = string(room);
    if global.__crystal_callback_room != undefined
        rm = string(global.__crystal_callback_room());
    __crystal_set_room(rm);
    var notf = __crystal_get_notification();
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
        notf = __crystal_get_notification();
    }
    return r;
}

function crystal_is_connected() {
    return __crystal_is_connected();
}

function crystal_is_connecting() {
    return __crystal_is_connecting();
}

function crystal_is_loggedin() {
    return __crystal_is_loggedin();
}

function crystal_get_ping() {
    return __crystal_get_ping();
}

function crystal_set_game_token(token) {
    return __crystal_set_game_token(token);
}

function crystal_disconnect() {
    return __crystal_disconnect();
}

function crystal_login(name, passw) {
    return __crystal_login(name, passw);
}

function crystal_login_with_token(name, token) {
    return __crystal_login_with_token(name, token);
}

function crystal_register(name, email, passw, repeat_passw) {
    return __crystal_register();
}

function crystal_get_player_id() {
    return __crystal_get_player_id();
}

function crystal_get_player_name() {
    return __crystal_get_player_name();
}

function crystal_set_variable(name, variable) {
    return __crystal_set_variable(name, __encode_variable(variable));
}

function crystal_remove_variable(name) {
    return __crystal_remove_variable(name);
}

function crystal_iter_other_players() {
	var ss = __crystal_iter_other_players();
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
    return __crystal_other_player_count();
}

function crystal_get_other_player(pid) {
    return __decode_player(__crystal_get_other_player(pid));
}

function crystal_get_other_player_name(name) {
    return __decode_player(__crystal_get_other_player_name(name));
}

function crystal_request_other_player_variable(pid, name, request) {
    return __crystal_request_other_player_variable(pid, name, request);
}

function crystal_p2p(target, mid, payload) {
    var s = string(array_length(payload));
    for (var i = 0; i < array_length(payload); i++)
        s += ";" + __encode_variable(payload[i]);
    return __crystal_p2p(target, mid, s);
}

function crystal_set_version(version) {
    return __crystal_set_version(version);
}

function crystal_get_version() {
    return __crystal_get_version();
}

function crystal_get_server_version() {
    return __crystal_get_server_version();
}

function crystal_set_session(session) {
    return __crystal_set_session(session);
}

function crystal_get_session() {
    return __crystal_get_session();
}

function crystal_get_open_playerini() {
    return __crystal_get_open_playerini();
}

function crystal_open_playerini(file) {
    return __crystal_open_playerini(file);
}

function crystal_close_playerini() {
    return __crystal_close_playerini();
}

function crystal_has_playerini(section, key) {
    return __crystal_has_playerini(section, key);
}

function crystal_get_playerini(section, key) {
    return __decode_variable(__crystal_get_playerini(section, key));
}

function crystal_set_playerini(section, key, value) {
    return __crystal_set_playerini(section, key, __encode_variable(value));
}

function crystal_remove_playerini(section, key) {
    return __crystal_remove_playerini(section, key);
}

function crystal_get_open_gameini() {
    return __crystal_get_open_gameini();
}

function crystal_open_gameini(file) {
    return __crystal_open_gameini(file);
}

function crystal_close_gameini() {
    return __crystal_close_gameini();
}

function crystal_has_gameini(section, key) {
    return __crystal_has_gameini(section, key);
}

function crystal_get_gameini(section, key) {
    return __decode_variable(__crystal_get_gameini(section, key));
}

function crystal_set_gameini(section, key, value) {
    return __crystal_set_gameini(section, key, __encode_variable(value));
}

function crystal_remove_gameini(section, key) {
    return __crystal_remove_gameini(section, key);
}

function crystal_has_achievement(aid) {
    return __crystal_has_achievement(aid);
}

function crystal_get_achievement(aid) {
    return __crystal_get_achievement(aid);
}

function crystal_has_reached_achievement(aid) {
    return __crystal_has_reached_achievement(aid);
}

function crystal_get_reached_achievement(aid) {
    return __crystal_get_reached_achievement(aid);
}

function crystal_reach_achievement(aid) {
    return __crystal_reach_achievement(aid);
}

function crystal_has_highscore(hid) {
    return __crystal_has_highscore(hid);
}

function crystal_get_highscore(hid) {
    return __crystal_get_highscore(hid);
}

function crystal_has_score_highscore(hid) {
    return __crystal_has_score_highscore(hid);
}

function crystal_get_score_highscore(hid) {
    return __crystal_get_score_highscore(hid);
}

function crystal_set_score_highscore(hid, score) {
    return __crystal_set_score_highscore(hid, score);
}

function crystal_create_sync(sync_type, kind) {
    return __crystal_create_sync(sync_type, kind);
}

function crystal_destroy_sync(sync) {
    return __crystal_destroy_sync(sync);
}

function crystal_set_variable_sync(sync, name, value) {
    return __crystal_set_variable_sync(sync, name, __encode_variable(value));
}

function crystal_remove_variable_sync(sync, name) {
    return __crystal_remove_variable_sync(sync, name);
}

function crystal_get_variable_other_sync(pid, sync, name) {
    return __crystal_get_variable_other_sync(pid, sync, name);
}

function crystal_iter_other_syncs() {
    var ss = __crystal_iter_other_syncs();
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
    return __crystal_is_player_admin(pid);
}

function crystal_get_player_admin(pid) {
    return __decode_administrator(__crystal_get_player_admin(pid));
}

function crystal_player_kick(pid, reason) {
    return __crystal_player_kick(pid, reason);
}

function crystal_player_ban(pid, reason, unban_time) {
    return __crystal_player_ban(pid, reason, unban_time);
}

function crystal_player_unban(pid) {
    return __crystal_player_unban(pid);
}

function crystal_logout() {
    return __crystal_logout();
}

function crystal_request_other_sync_variable(pid, slot, name, request) {
    return __crystal_request_other_sync_variable(pid, slot, name, request);
}

function crystal_fetch_bdb(name) {
    return __crystal_fetch_bdb(name);
}

function crystal_set_bdb(name, data) {
    return __crystal_set_bdb(name, buffer_base64_encode(data, 0, buffer_tell(data)));
}

function crystal_get_incoming_friends() {
    var s = string_split(__crystal_get_incoming_friends(), ":");
    var r = [];
    var sz = real(s[0]);
    for (var i = 0; i < sz; i++)
        array_push(r, real(s[i + 1]));
    return r;
}

function crystal_get_outgoing_friends() {
    var s = string_split(__crystal_get_outgoing_friends(), ":");
    var r = [];
    var sz = real(s[0]);
    for (var i = 0; i < sz; i++)
        array_push(r, real(s[i + 1]));
    return r;
}

function crystal_get_friends() {
    var s = string_split(__crystal_get_friends(), ":");
    var r = [];
    var sz = real(s[0]);
    for (var i = 0; i < sz; i++)
        array_push(r, real(s[i + 1]));
    return r;
}

function crystal_send_outgoing_friend(pid) {
    return __crystal_send_outgoing_friend(pid);
}

function crystal_remove_outgoing_friend(pid) {
    return __crystal_remove_outgoing_friend(pid);
}

function crystal_deny_incoming_friend(pid) {
    return __crystal_deny_incoming_friend(pid);
}

function crystal_accept_incoming_friend(pid) {
    return __crystal_accept_incoming_friend(pid);
}

function crystal_remove_friend(pid) {
    return __crystal_remove_friend(pid);
}

/*
function crystal_() {
    return __crystal_();
}
*/

function __decode_administrator(s) {
    s = string_split(s, ":");
    var a = new CrystalAdministrator();
    a.can_ban = bool(real(s[0]));
    a.can_unban = bool(real(s[1]));
    a.can_kick = bool(real(s[2]));
    return a;

}

function __decode_player(s) {
	//show_debug_message(s);
    s = string_split(s, ":");
	//show_debug_message(s);
    var p = new CrystalPlayer();
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
    var sy = new CrystalSync();
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
	var si = new CrystalSyncIter();
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
