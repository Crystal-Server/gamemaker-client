enum CompressionType {
    None = 0,
    Zstd = 1,
    Gzip = 2,
    Zlib = 3,
}

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
    Semi = 1,
    Full = 2,
}

enum P2PCode {
    AllGame = -5,
    CurrentSession = -6,
    CurrentRoom = -7,
}

global.__socket = undefined;
global.__game_id = "";
global.__is_connected = false;
global.__is_loggedin = false;
global.__is_connecting = false;
global.__version = 0.0;
global.__session = "";
global.__game_token = "";
global.__current_room = "";
global.__async_network_id = -1;

global.__enable_compression = false;

global.__script_room = undefined;
global.__script_p2p = undefined;
global.__script_register = undefined;
global.__script_login = undefined;
global.__script_banned = undefined;
global.__script_kicked = undefined;
global.__script_disconnected = undefined;
global.__script_login_token = undefined;
 
global.__player_id = -1;
global.__player_name = "";
global.__player_save = {};
global.__player_open_save = "";

global.__game_save = {};
global.__game_open_save = "";
global.__game_achievements = {};
global.__game_highscores = {};
global.__game_adminstrators = {};
global.__game_version = 0.0;

global.__players = {};
global.__players_logout = [];
global.__players_queue = {};
global.__variables = {};
global.__syncs = [];
global.__syncs_remove = [];
        
global.__ping = 0.0;
global.__last_ping = undefined;
        
global.__buffered_data = [];
global.__new_sync_queue = [];
global.__update_vari = [];
global.__update_gameini = [];
global.__update_playerini = [];
global.__callback_other_vari = [];
global.__call_disconnected = true;
global.__compression = CompressionType.None;
global.__is_queued = false;

global.__expected_packet_size = -1;
global.__expected_compression_type = undefined;
global.__streamed_data = undefined;
global.__buffered_receiver = undefined;
global.__buffered_receiver_size = 0;
global.__handshake_completed = false;

function _buf_write_leb_u64(buf, val) {
    while true {
        var n = val & 0x7f;
        val = val >> 7;
        if val == 0 {
            buffer_write(buf, buffer_u8, n);
            break;
        } else {
            buffer_write(buf, buffer_u8, n | 0x80);
        }
    }
}

function _buf_write_i64(buf, val) {
    buffer_write(buf, buffer_u64, int64(val)); // Hacky conversion, it mostly works
}

function _buf_write_bool(buf, val) {
    buffer_write(buf, buffer_u8, real(val));
}

function _buf_write_string(buf, val) {
    _buf_write_leb_u64(buf, string_byte_length(val));
    buffer_write(buf, buffer_text, val);
}

function _buf_write_bytes(buf, val) {
    _buf_write_leb_u64(buf, buffer_tell(val));
    buffer_resize(buf, buffer_tell(buf) + buffer_tell(val));
    buffer_copy(val, 0, buffer_tell(val), buf, buffer_tell(buf));
    buffer_seek(buf, buffer_seek_relative, buffer_tell(val));
}

function _buf_read_leb_u64(buf) {
    var num = 0;
    var shift = 0;
    while true {
        var val = buffer_read(buf, buffer_u8);
        num |= (val & 0x7f) << shift;
        if val & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    return num;
}

function _buf_read_i64(buf) {
	return int64(buffer_read(buf, buffer_u64)); // Least hacky GameMaker thing ever
}

function _buf_read_bool(buf) {
    return buffer_read(buf, buffer_u8) != 0;
}

function _buf_read_string(buf) {
    var size = _buf_read_leb_u64(buf);
	if size == 0
		return "";
    var rbuf = buffer_create(size, buffer_fixed, 1);
    buffer_copy(buf, buffer_tell(buf), size, rbuf, 0);
    buffer_seek(buf, buffer_seek_relative, size);
    var res = buffer_read(rbuf, buffer_text);
    buffer_delete(rbuf);
    return res;
}

function _buf_read_bytes(buf) {
    var size = _buf_read_leb_u64(buf);
    var rbuf = buffer_create(size, buffer_fixed, 1);
    buffer_copy(buf, buffer_tell(buf), size, rbuf, 0);
    buffer_seek(buf, buffer_seek_relative, size);
    return rbuf;
}

function _buf_write_map(buf, val) {
    _buf_write_leb_u64(buf, ds_map_size(val));
    var key = ds_map_find_first(val);
    while key != undefined {
        _buf_write_string(buf, string(key));
        _buf_write_value(buf, ds_map_find_value(val, key));
        key = ds_map_find_next(val, key);
    }
}

function _buf_write_list(buf, val) {
    _buf_write_leb_u64(buf, ds_list_size(val));
    for (var i = 0; i < ds_list_size(val); i++) {
        _buf_write_value(buf, ds_list_find_value(val, i));
    }
}

function _buf_read_struct(buf) {
    var map = {};
    var size = _buf_read_leb_u64(buf);
    repeat size {
        var key = _buf_read_string(buf);
        struct_set(map, key, _buf_read_value(buf));
    }
    return map;
}

function _buf_read_array(buf) {
    var list = [];
    var size = _buf_read_leb_u64(buf);
    repeat size {
        array_push(list, _buf_read_value(buf));
    }
    return list;
}

function _buf_write_struct(buf, val) {
    var names = struct_get_names(val);
    _buf_write_leb_u64(buf, array_length(names));
    for (var i = 0; i < array_length(names); i++) {
        var key = names[i];
        _buf_write_string(buf, string(key));
        _buf_write_value(buf, struct_get(val, key));
    }
}

function _buf_write_array(buf, val) {
    _buf_write_leb_u64(buf, array_length(val));
    for (var i = 0; i < array_length(val); i++) {
        _buf_write_value(buf, val[i]);
    }
}

function _buf_write_value(buf, val) {
    switch typeof(val) {
        case "undefined":
        case "null":
            buffer_write(buf, buffer_u8, 0);
            break;
        case "bool":
            buffer_write(buf, buffer_u8, 1);
            _buf_write_bool(buf, val);
            break;
        case "int32":
        case "int64":
            buffer_write(buf, buffer_u8, 2);
            _buf_write_i64(buf, int64(val));
            break;
        case "number":
            buffer_write(buf, buffer_u8, 3);
            buffer_write(buf, buffer_f64, val);
            break;
        case "string":
            buffer_write(buf, buffer_u8, 4);
            _buf_write_string(buf, val);
            break;
        case "array":
            buffer_write(buf, buffer_u8, 5);
            _buf_write_array(buf, val);
            break;
        case "struct":
            buffer_write(buf, buffer_u8, 6);
            _buf_write_struct(buf, val);
            break;
        case "ref":
            if buffer_exists(val) {
                buffer_write(buf, buffer_u8, 7);
                _buf_write_bytes(buf, val);
            } else if ds_exists(val, ds_type_list) {
                buffer_write(buf, buffer_u8, 5);
                _buf_write_list(buf, val);
            } else if ds_exists(val, ds_type_map) {
                buffer_write(buf, buffer_u8, 6);
                _buf_write_map(buf, val);
            } else {
                show_debug_message("Unknown value type: {0}, defaulting to undefined", val);
                buffer_write(buf, buffer_u8, 0);
            }
            break;
        default:
            show_debug_message("Unknown value type: {0}, defaulting to undefined", val);
            buffer_write(buf, buffer_u8, 0);
            break;
    }
}

function _buf_read_value(buf) {
    switch buffer_read(buf, buffer_u8) {
        case 1:
            return _buf_read_bool(buf);
        case 2:
            return _buf_read_i64(buf);
        case 3:
            return buffer_read(buf, buffer_f64);
        case 4:
            return _buf_read_string(buf);
        case 5:
            return _buf_read_array(buf);
        case 6:
            return _buf_read_struct(buf);
        case 7:
            return _buf_read_bytes(buf);
        default:
            return undefined;
    }
}

function _buf_write_syncs(buf, val) {
    _buf_write_leb_u64(buf, array_length(val));
    for (var i = 0; i < array_length(val); i++) {
        var sync = val[i];
        if sync == undefined {
            _buf_write_bool(buf, false);
        } else {
            _buf_write_bool(buf, true);
            buffer_write(buf, buffer_s16, sync.kind);
            buffer_write(buf, buffer_u8, sync.type);
            _buf_write_struct(buf, sync.variables);
        }
    }
}

function _buf_read_syncs(buf) {
    var val = [];
    var size = _buf_read_leb_u64(buf);
    repeat size {
        if _buf_read_bool(buf) {
            var sync = _create_sync();
            sync.kind = buffer_read(buf, buffer_s16);
            sync.type = buffer_read(buf, buffer_u8);
            sync.variables = _buf_read_struct(buf);
            array_push(val, sync);
        } else {
            array_push(val, undefined);
        }
    }
    return val;
}

function _buf_read_achievements(buf) {
    var val = {};
    var size = _buf_read_leb_u64(buf);
    repeat size {
        var name = buffer_read(buf, buffer_u64);
        var achi = _create_achievement();
        achi.id = _buf_read_leb_u64(buf);
        achi.name = _buf_read_string(buf);
        achi.description = _buf_read_string(buf);
        var players = {};
        var psize = _buf_read_leb_u64(buf);
        repeat psize {
            var pname = buffer_read(buf, buffer_u64);
            players[$ pname] = _buf_read_i64(buf);
        }
        achi.players = players;
        val[$ name] = achi;
    }
    return val;
}

function _buf_read_highscores(buf) {
    var val = {};
    var size = _buf_read_leb_u64(buf);
    repeat size {
        var name = buffer_read(buf, buffer_u64);
        var hsc = _create_highscore();
        hsc.id = _buf_read_leb_u64(buf);
        hsc.name = _buf_read_string(buf);
        var scores = {};
        var psize = _buf_read_leb_u64(buf);
        repeat psize {
            var sname = buffer_read(buf, buffer_u64);
            scores[$ sname] = buffer_read(buf, buffer_f64);
        }
        hsc.scores = scores;
        val[$ name] = hsc;
    }
    return val;
}

function _buf_read_administrators(buf) {
    var val = {};
    var size = _buf_read_leb_u64(buf);
    repeat size {
        var name = buffer_read(buf, buffer_u64);
        var admin = _create_administrator();
        admin.id = _buf_read_leb_u64(buf);
        admin.can_kick = _buf_read_bool(buf);
        admin.can_ban = _buf_read_bool(buf);
        admin.can_unban = _buf_read_bool(buf);
        val[$ name] = admin;
    }
    return val;
}

function _iter_buffered_data() {
    var temp = [];
    array_copy(temp, 0, global.__buffered_data, 0, array_length(global.__buffered_data));
    array_delete(global.__buffered_data, 0, array_length(global.__buffered_data));
    for (var i = 0; i < array_length(temp); i++) {
        _buf_send(temp[i]);
    }
    array_delete(temp, 0, array_length(temp));
}

function _buf_send(buf) {
    if global.__is_connected {
        var nbuf = buffer_create(0, buffer_grow, 1);
        if global.__compression == CompressionType.Zlib {
            var pos = buffer_tell(buf);
            var tnbuf = buffer_compress(buf, 0, pos);
            buffer_seek(tnbuf, buffer_seek_end, 0); // Maybe?
            buffer_delete(buf);
            buf = tnbuf;
        }
		var event = buffer_peek(buf, 0, buffer_u8);
        _buf_write_leb_u64(nbuf, buffer_tell(buf));
        buffer_write(nbuf, buffer_u8, global.__compression);
        buffer_resize(nbuf, buffer_tell(nbuf) + buffer_tell(buf));
        buffer_copy(buf, 0, buffer_tell(buf), nbuf, buffer_tell(nbuf));
        var r = network_send_raw(global.__socket, nbuf, buffer_tell(nbuf) + buffer_tell(buf));
        buffer_delete(buf);
        buffer_delete(nbuf);
        //show_debug_message("{1}->Sent {0} byte(s)", r, event);
		if r < 0 {
			global.__is_connected = false;
            if global.__call_disconnected {
                if global.__script_disconnected != undefined
                    global.__script_disconnected();
                global.__call_disconnected = false;
            }
            _crystal_clear_disconnected();
		}
    } else {
        array_push(global.__buffered_data, buf);
    }
}

function _crystal_verify_init() {
    return variable_global_exists("__crystal_object");
}

function _crystal_partial_clear_disconnected() {
    global.__player_name = "";
	global.__player_id = -1;
	global.__player_save = {};
    global.__player_open_save = "";
	global.__players = {};
    global.__players_queue = {};
	global.__is_loggedin = false;
	array_delete(global.__players_logout, 0, array_length(global.__players_logout));
	for (var i = 0; i < array_length(global.__buffered_data); i++) {
        buffer_delete(global.__buffered_data[i]);
    }
	array_delete(global.__buffered_data, 0, array_length(global.__buffered_data));
    array_delete(global.__new_sync_queue, 0, array_length(global.__new_sync_queue));
    array_delete(global.__update_vari, 0, array_length(global.__update_vari));
    array_delete(global.__update_playerini, 0, array_length(global.__update_playerini));
    array_delete(global.__callback_other_vari, 0, array_length(global.__callback_other_vari));
}

function _crystal_clear_disconnected() {
    _crystal_partial_clear_disconnected();
    global.__game_save = {};
    global.__game_open_save = "";
    global.__game_achievements = {};
    global.__game_highscores = {};
    global.__game_adminstrators = {};
    array_delete(global.__update_gameini, 0, array_length(global.__update_gameini));
    if global.__socket != undefined { // SAFETY: Don't try to destroy if it already got destroyed
        network_destroy(global.__socket);
        global.__socket = undefined;
    }
    global.__buffered_receiver_size = 0;
    if global.__buffered_receiver != undefined { // SAFETY: Don't try to destroy if it already got destroyed
        buffer_delete(global.__buffered_receiver);
        global.__buffered_receiver = undefined;
    }
    global.__last_ping = undefined;
}

function crystal_init(game_id) {
    if !variable_global_exists("__crystal_object") {
        global.__crystal_object = id;
        global.__game_id = game_id;
        if os_browser == browser_not_a_browser
            global.__socket = network_create_socket(network_socket_tcp);
        else
            global.__socket = network_create_socket(network_socket_ws);
    }
}

function crystal_step() {
    if _crystal_verify_init() {
        if global.__socket != undefined {
            if !global.__is_connected && global.__is_loggedin
                global.__is_loggedin = false;
            if global.__is_connected {
                if global.__last_ping != undefined {
                    if (get_timer() - global.__last_ping) / 1000000 > 60 * 2 {
                        global.__is_connected = false;
                        if global.__call_disconnected {
                            if global.__script_disconnected != undefined
                                global.__script_disconnected();
                            global.__call_disconnected = false;
                        }
                        _crystal_clear_disconnected();
                    }
                }
                var r = _get_room();
                var player_keys = struct_get_names(global.__players);
                for (var i = 0; i < array_length(player_keys); i++) {
                    var key = player_keys[i];
                    var player = global.__players[$ key];
                    var player_logged_out = array_contains(global.__players_logout, key);
                    var is_all_undefined = true;
                    for (var ii = 0; ii < array_length(player.syncs); ii++) {
                        var sync = player.syncs[ii];
                        if sync != undefined {
                            is_all_undefined = false;
                            if sync.event == SyncEvent.End
                                player.syncs[ii] = undefined;
                            if sync.type == CreateSync.Once
                                sync.event = SyncEvent.End;
                            if r != player.room || player_logged_out
                                sync.event = SyncEvent.End;
                        }
                    }
                    _iter_missing_data(key);
                    if is_all_undefined && player_logged_out {
                        struct_remove(global.__players, key);
    					for (var ii = 0; ii < array_length(global.__players_logout); ii++) {
    						if global.__players_logout[ii] == key {
    							array_delete(global.__players_logout, ii, 1);
    							break;
    						}
    					}
                        var players_queue_keys = struct_get_names(global.__players_queue);
                        for (var ii = 0; ii < array_length(players_queue_keys); ii++) {
                            if global.__players_queue[$ players_queue_keys[ii]] == key {
                                struct_remove(global.__players_queue, players_queue_keys[ii]);
                                break;
                            }
                        }
                    }
                }
                _iter_buffered_data();
                if r != global.__current_room {
                    var b = buffer_create(0, buffer_grow, 1);
                    buffer_write(b, buffer_u8, 11);
                    _buf_write_string(b, r);
                    _buf_send(b);
                    global.__current_room = r;
                }
                if array_length(global.__syncs) > 0 || array_length(global.__syncs_remove) > 0 {
                    var b = buffer_create(0, buffer_grow, 1);
                    var insts_iter = 0;
                    for (var i = 0; i < array_length(global.__syncs_remove); i++) {
                        buffer_write(b, buffer_s16, global.__syncs_remove[i]);
                        _buf_write_bool(b, true);
                        insts_iter++;
                    }
    				array_delete(global.__syncs_remove, 0, array_length(global.__syncs_remove));
                    for (var i = 0; i < array_length(global.__syncs); i++) {
                        var sync = global.__syncs[i];
                        if sync != undefined {
                            if array_length(sync.to_sync) > 0 {
                                buffer_write(b, buffer_s16, i);
                                _buf_write_bool(b, false);
                                _buf_write_leb_u64(b, array_length(sync.to_sync));
                                for (var ii = 0; ii < array_length(sync.to_sync); ii++) {
                                    var to_sync = sync.to_sync[ii];
                                    _buf_write_string(b, to_sync);
                                    if struct_exists(sync.variables, to_sync)
                                        _buf_write_value(b, sync.variables[$ to_sync])
                                    else
                                        buffer_write(b, buffer_u8, 0xff);
                                }
                                sync.to_sync = [];
                                insts_iter++;
                            }
                        }
                    }
                    if insts_iter > 0 {
                        var tb = buffer_create(0, buffer_grow, 1);
                        buffer_write(tb, buffer_u8, 13);
                        _buf_write_leb_u64(tb, insts_iter);
                        buffer_resize(tb, buffer_tell(tb) + buffer_tell(b));
                        buffer_copy(b, 0, buffer_tell(b), tb, buffer_tell(tb));
    					buffer_seek(tb, buffer_seek_relative, buffer_tell(b));
                        _buf_send(tb);
                    }
                    buffer_delete(b);
                }
                if array_length(global.__new_sync_queue) > 0 {
                    for (var i = 0; i < array_length(global.__new_sync_queue); i++) {
                        var s = global.__new_sync_queue[i];
                        var b = buffer_create(0, buffer_grow, 1);
                        buffer_write(b, buffer_u8, 12);
                        buffer_write(b, buffer_u16, s.slot);
                        buffer_write(b, buffer_s16, s.kind);
                        buffer_write(b, buffer_u8, s.type);
                        _buf_write_struct(b, global.__syncs[s.slot].variables);
                        _buf_send(b);
                        global.__syncs[s.slot].to_sync = [];
                    }
                    array_delete(global.__new_sync_queue, 0, array_length(global.__new_sync_queue));
                }
                if array_length(global.__update_vari) > 0 {
                    var b = buffer_create(0, buffer_grow, 1);
                    buffer_write(b, buffer_u8, 7);
                    _buf_write_leb_u64(b, array_length(global.__update_vari));
                    for (var i = 0; i < array_length(global.__update_vari); i++) {
                        var u = global.__update_vari[i];
                        _buf_write_string(b, u.name);
                        if u.removed
                            buffer_write(b, buffer_u8, 0xff);
                        else
                            _buf_write_value(b, u.value);
                    }
                    array_delete(global.__update_vari, 0, array_length(global.__update_vari));
    				_buf_send(b);
                }
                if array_length(global.__update_gameini) > 0 {
                    var b = buffer_create(0, buffer_grow, 1);
                    buffer_write(b, buffer_u8, 9);
                    _buf_write_leb_u64(b, array_length(global.__update_gameini));
                    for (var i = 0; i < array_length(global.__update_gameini); i++) {
                        var u = global.__update_gameini[i];
                        _buf_write_string(b, u.name);
                        if u.removed
                            buffer_write(b, buffer_u8, 0xff);
                        else
                            _buf_write_value(b, u.value);
                    }
                    array_delete(global.__update_gameini, 0, array_length(global.__update_gameini));
                    _buf_send(b);
                }
                if array_length(global.__update_playerini) > 0 {
                    var b = buffer_create(0, buffer_grow, 1);
                    buffer_write(b, buffer_u8, 10);
                    _buf_write_leb_u64(b, array_length(global.__update_playerini));
                    for (var i = 0; i < array_length(global.__update_playerini); i++) {
                        var u = global.__update_playerini[i];
                        _buf_write_string(b, u.name);
                        if u.removed
                            buffer_write(b, buffer_u8, 0xff);
                        else
                            _buf_write_value(b, u.value);
                    }
                    array_delete(global.__update_playerini, 0, array_length(global.__update_playerini));
                    _buf_send(b);
                }
            }
        }
    }
}

function crystal_async_networking() {
	//show_debug_message(json_encode(async_load));
    if _crystal_verify_init() {
        if async_load[? "id"] == global.__socket {
            switch async_load[? "type"] {
                case network_type_non_blocking_connect:
                    global.__is_connecting = false;
                    global.__is_connected = async_load[? "succeeded"] == true;
                    break;
                case network_type_disconnect:
                    global.__is_connected = false;
                    if global.__call_disconnected {
                        if global.__script_disconnected != undefined
                            global.__script_disconnected();
                        global.__call_disconnected = false;
                    }
                    _crystal_clear_disconnected();
                    break;
                case network_type_data:
                    var buf = async_load[? "buffer"];
                    var size = async_load[? "size"];
                    if global.__buffered_receiver == undefined
                        global.__buffered_receiver = buffer_create(size, buffer_grow, 1);
                    buffer_resize(global.__buffered_receiver, global.__buffered_receiver_size + size);
                    buffer_copy(buf, 0, size, global.__buffered_receiver, global.__buffered_receiver_size);
                    global.__buffered_receiver_size += size;
                    while true {
                        if global.__buffered_receiver_size == 0 {
                            //show_debug_message("Receiver payload finished. Waiting for more packets.");
                            break;
                        }
                        var offset = buffer_tell(global.__buffered_receiver);
                        if global.__expected_packet_size == -1 {
                            global.__expected_packet_size = 0;
                            var shift = 0;
                            var ret = false;
                            while true {
                                if buffer_tell(global.__buffered_receiver) + 1 > global.__buffered_receiver_size {
                                    //show_debug_message("Incomplete LEB128 header, waiting...");
                                    ret = true;
                                    break;
                                }
                                var val = buffer_read(global.__buffered_receiver, buffer_u8);
                                global.__expected_packet_size |= (val & 0x7f) << shift;
                                if val & 0x80 == 0
                                    break;
                                shift += 7;
                            }
                            if ret {
                                global.__expected_packet_size = -1;
                                global.__expected_compression_type = -1;
                                buffer_seek(global.__buffered_receiver, buffer_seek_start, offset);
                                break;
                            }
                            if buffer_tell(global.__buffered_receiver) + 1 > global.__buffered_receiver_size {
                                //show_debug_message("Incomplete decompression header, waiting...");
                                global.__expected_packet_size = -1;
                                global.__expected_compression_type = -1;
                                buffer_seek(global.__buffered_receiver, buffer_seek_start, offset);
                                break;
                            } else
                                global.__expected_compression_type = buffer_read(global.__buffered_receiver, buffer_u8);
                        }
                        if global.__buffered_receiver_size - buffer_tell(global.__buffered_receiver) >= global.__expected_packet_size - 1 && global.__expected_packet_size > 0 {
                            global.__streamed_data = buffer_create(global.__expected_packet_size - 1, buffer_fixed, 1);
                            buffer_copy(global.__buffered_receiver, buffer_tell(global.__buffered_receiver), global.__expected_packet_size, global.__streamed_data, 0);
                            var copy_offset = buffer_tell(global.__buffered_receiver) + (global.__expected_packet_size - 1);
                            var copy_size = buffer_get_size(global.__buffered_receiver) - copy_offset;
                            var temp = buffer_create(copy_size, buffer_fixed, 1);
                            buffer_copy(global.__buffered_receiver, copy_offset, copy_size, temp, 0);
                            buffer_copy(temp, 0, copy_size, global.__buffered_receiver, offset);
                            buffer_delete(temp);
                            global.__buffered_receiver_size -= (global.__expected_packet_size - 1) + (buffer_tell(global.__buffered_receiver) - offset);
                            buffer_resize(global.__buffered_receiver, global.__buffered_receiver_size);
                            buffer_seek(global.__buffered_receiver, buffer_seek_start, offset);
                            switch global.__expected_compression_type {
                                case CompressionType.None:
                                    // This won't execute anything else.
                                    break;
                                case CompressionType.Zstd:
                                    show_debug_message("Unsupported compression \"ZSTD\"");
                                    break;
                                case CompressionType.Gzip:
                                    show_debug_message("Unsupported compression \"GZIP\"");
                                    break;
                                case CompressionType.Zlib:
                                    var decompressed = buffer_decompress(global.__streamed_data);
                                    buffer_delete(global.__streamed_data);
                                    global.__streamed_data = decompressed;
                                    break;
                            }
                            global.__expected_packet_size = -1;
                            global.__expected_compression_type = -1;
                            buffer_seek(global.__streamed_data, buffer_seek_start, 0);
                            _handle_packet(global.__streamed_data);
                            buffer_delete(global.__streamed_data);
                        } else {
                            //show_debug_message("Incomplete data payload, waiting... Expected {0} byte(s), but got {1} byte(s)", global.__expected_packet_size, global.__buffered_receiver_size - buffer_tell(global.__buffered_receiver));
                            break;
                        }
                    }
                    break;
            }
        }
    }
}

function _iter_missing_data(pid) {
    if struct_exists(global.__players, pid) && struct_exists(global.__players_queue, pid) {
        var player = global.__players[$ pid];
        var pq = global.__players_queue[$ pid];
        var pq_vari_names = struct_get_names(pq.variables);
        for (var i = 0; i < array_length(pq_vari_names); i++) {
            var vq = pq.variables[$ pq_vari_names[i]];
            if vq.type == VariableQueueType.Set
                player.variables[$ vq.name] = vq.value;
            else if vq.type == VariableQueueType.Remove
                struct_remove(player.variables, vq.name);
        }
        pq.variables = {};
        for (var i = 0; i < array_length(player.syncs); i++) {
            if player.syncs[i] == undefined && struct_exists(pq.syncs_new, i) {
                var sn = pq.syncs_new[$ i];
                var s = _create_sync();
                s.event = SyncEvent.New;
                s.kind = sn.kind;
                s.type = sn.type;
                s.variables = sn.variables;
                player.syncs[i] = s;
            }
            if player.syncs[i] != undefined {
                if struct_exists(pq.syncs_new, i)
                    struct_remove(pq.syncs_new, i);
                if struct_exists(pq.syncs, i) {
                    var is = pq.syncs[$ i];
                    var is_names = struct_get_names(is);
                    for (var ii = 0; ii < array_length(is_names); ii++) {
                        var vq = is[$ is_names[ii]];
                        if vq.type == VariableQueueType.Set
                            player.syncs[i].variables[$ vq.name] = vq.value;
                        else if vq.type == VariableQueueType.Remove
                            struct_remove(player.syncs[i].variables, vq.name);
                    }
                    struct_remove(pq.syncs, i);
                }
                if struct_exists(pq.syncs_remove, i) {
                    player.syncs[i].event = SyncEvent.End;
                    struct_remove(pq.syncs_remove, i);
                }
            }
        }
        struct_remove(global.__players_queue, pid);
    }
}

function _create_synciter() {
    return {
        "event": SyncEvent.New,
        "kind": -1,
        "variables": {},
        "player_id": -1,
        "player_name": "",
        "slot": -1,
    };
}

function _create_administrator() {
    return {
        "id": -1,
        "can_kick": false,
        "can_ban": false,
        "can_unban": false,
    };
}

function _create_sync() {
    return {
        "kind": -1,
        "type": CreateSync.Full,
        "event": SyncEvent.New,
        "variables": {},
        "to_sync": [],
    };
}

function _create_newsyncqueue() {
    return {
        "slot": -1,
        "kind": -1,
        "type": CreateSync.Full,
    };
}

function _create_updatevari() {
    return {
        "removed": false,
        "name": "",
        "value": undefined,
    };
} 

function _create_player() {
    return {
        "id": -1,
        "name": "",
        "room": "",
        "syncs": [],
        "variables": {},
    };
}

function _create_achievement() {
    return {
        "id": -1,
        "name": "",
        "description": "",
        "players": {},
    };
}

function _create_highscore() {
    return {
        "id": -1,
        "name": "",
        "scores": {},
    };
}

function _create_playerqueue() {
    return {
        "variables": {},
        "syncs": {},
        "syncs_remove": {},
        "syncs_new": {},
    };
}

function _create_syncnew() {
    return {
        "slot": -1,
        "type": CreateSync.Full,
        "kind": -1,
        "variables": {},
    };
}

function _create_variablequeue() {
    return {
        "type": VariableQueueType.Null,
        "name": "",
        "value": undefined,
    }
}

function _handle_packet(buf) {
    //show_debug_message("Handling event {0}...", buffer_peek(buf, 0, buffer_u8));
    switch buffer_read(buf, buffer_u8) {
        case 0: // Registration
            var code = buffer_read(buf, buffer_u8);
            if global.__script_register != undefined
                global.__script_register(code);
            break;
        case 1: // Login
            code = buffer_read(buf, buffer_u8);
            var token = "";
            switch code {
                case LoginResult.OK:
                    global.__is_loggedin = true;
                    global.__player_id = _buf_read_leb_u64(buf);
                    global.__player_name = _buf_read_string(buf);
                    if _buf_read_bool(buf)
                        token = _buf_read_string(buf);
                    global.__player_save = _buf_read_struct(buf);
                    break;
                case LoginResult.GameBan:
                    var reason = _buf_read_string(buf);
                    var unban_time = _buf_read_i64(buf);
                    if global.__script_login != undefined
                        global.__script_login(code, unban_time, reason);
                    break;
            }
            if code != LoginResult.GameBan && global.__script_login != undefined
                global.__script_login(code);
            if string_length(token) != 0 && global.__script_login_token != undefined
                global.__script_login_token(token);
            break;
        case 2: // User logged in
            var _player_id = _buf_read_leb_u64(buf);
            var name = _buf_read_string(buf);
            var _room = _buf_read_string(buf);
            var variables = _buf_read_struct(buf);
            var syncs = _buf_read_syncs(buf);
            
            _iter_missing_data(_player_id);
            
            var player = _create_player();
            player.id = _player_id;
            player.name = name;
            player.variables = variables;
            player.syncs = syncs;
            player.room = _room;
            global.__players[$ _player_id] = player;
            break;
        case 3: // User logged out
            array_push(global.__players_logout, _buf_read_leb_u64(buf));
            break;
        case 4: // Sync game info
            global.__game_save = _buf_read_struct(buf);
            global.__game_achievements = _buf_read_achievements(buf);
            global.__game_highscores = _buf_read_highscores(buf);
            global.__game_adminstrators = _buf_read_administrators(buf);
            global.__game_version = buffer_read(buf, buffer_f64);
            global.__handshake_completed = true;
            global.__last_ping = get_timer();
            break;
        case 5: // P2P
            _player_id = _buf_read_leb_u64(buf);
            var message_id = buffer_read(buf, buffer_u16);
            var arr = _buf_read_array(buf);
            if global.__script_p2p != undefined
                global.__script_p2p(message_id, _player_id, arr);
            break;
        case 6: // Sync player variable
            _player_id = _buf_read_leb_u64(buf);
            _iter_missing_data(_player_id);
            var amount = _buf_read_leb_u64(buf);
            for (var i = 0; i < amount; i++) {
                name = _buf_read_string(buf);
                var remove_vari = buffer_read(buf, buffer_u8) == 0xff;
                buffer_seek(buf, buffer_seek_relative, -1);
                var value = _buf_read_value(buf);
                if struct_exists(global.__players, _player_id) {
                    player = global.__players[$ _player_id];
                    if remove_vari
                        struct_remove(player.variables, name);
                    else
                        player.variables[$ name] = value;
                } else {
                    _check_players_queue(_player_id);
                    var pq = global.__players_queue[$ _player_id];
                    var vq = _create_variablequeue();
                    vq.name = name;
                    if remove_vari
                        vq.type = VariableQueueType.Remove;
                    else
                        vq.type = VariableQueueType.Set;
                    vq.value = value;
                    pq.variables[$ name] = vq;
                }
            }
            break;
        case 7: // Ping
            switch buffer_read(buf, buffer_u8) {
                case 0:
                    var wb = buffer_create(0, buffer_grow, 1);
                    buffer_write(wb, buffer_u8, 8);
                    _buf_send(wb);
                    break;
                case 1:
                    global.__ping = buffer_read(buf, buffer_f64);
                    global.__last_ping = get_timer();
                    break;
            }
            break;
        case 8: // Clear players
            global.__players = {};
            global.__players_queue = {};
            break;
        case 9: // Game ini write
            var size = _buf_read_leb_u64(buf);
            repeat size {
                name = _buf_read_string(buf);
                var remove_vari = buffer_read(buf, buffer_u8) == 0xff;
                buffer_seek(buf, buffer_seek_relative, -1);
                var vari = _buf_read_value(buf);
                if remove_vari
                    struct_remove(global.__game_save, name);
                else
                    global.__game_save[$ name] = vari;
            }
            break;
        case 10: // New sync
            _player_id = _buf_read_leb_u64(buf);
            var slot = buffer_read(buf, buffer_u16);
            var kind = buffer_read(buf, buffer_s16);
            var type = buffer_read(buf, buffer_u8);
            variables = _buf_read_struct(buf);
            if struct_exists(global.__players, _player_id) {
                _iter_missing_data(_player_id);
                player = global.__players[$ _player_id];
                while array_length(player.syncs) <= slot
                    array_push(player.syncs, undefined);
                var sync = _create_sync();
                sync.event = SyncEvent.New;
                sync.kind = kind;
                sync.type = type;
                sync.variables = variables;
                player.syncs[slot] = sync;
            } else {
                _check_players_queue(_player_id);
                var pq = global.__players_queue[$ _player_id];
                var sn = _create_syncnew();
                sn.slot = slot;
                sn.kind = kind;
                sn.type = type;
                sn.variables = variables;
                pq.syncs_new[$ slot] = sn;
            }
            break;
        case 11: // Player changed rooms
            _player_id = _buf_read_leb_u64(buf);
            _room = _buf_read_string(buf);
            if struct_exists(global.__players, _player_id) {
                _iter_missing_data(_player_id);
                global.__players[$ _player_id].room = _room;
            }
            break;
        case 12: // Update sync
            _player_id = _buf_read_leb_u64(buf);
            _iter_missing_data(_player_id);
            syncs = struct_exists(global.__players, _player_id) ? global.__players[$ _player_id].syncs : [];
            amount = _buf_read_leb_u64(buf);
            for (var i = 0; i < amount; i++) {
                slot = buffer_read(buf, buffer_u16);
                if _buf_read_bool(buf) { // Remove sync
                    if slot >= array_length(syncs) || syncs[slot] == undefined {
                        _check_players_queue(_player_id);
                        global.__players_queue[$ _player_id].syncs_remove[$ slot] = 0;
                    } else
                        global.__players[$ _player_id].syncs[slot].event = SyncEvent.End;
                } else { // Update variables
                    var amount1 = _buf_read_leb_u64(buf);
                    for (var ii = 0; ii < amount1; ii++) {
                        name = _buf_read_string(buf);
                        var remove_vari = buffer_read(buf, buffer_u8) == 0xff;
                        buffer_seek(buf, buffer_seek_relative, -1);
                        var value = _buf_read_value(buf);
                        if slot >= array_length(syncs) || syncs[slot] == undefined {
                            _check_players_queue(_player_id);
                            var pq = global.__players_queue[$ _player_id];
                            if !struct_exists(pq.syncs, slot)
                                pq.syncs[$ slot] = {};
                            var vq = _create_variablequeue();
                            vq.name = name;
                            vq.type = remove_vari ? VariableQueueType.Remove : VariableQueueType.Set;
                            vq.value = value;
                            pq.syncs[$ slot][$ name] = vq;
                        } else if remove_vari
                            struct_remove(syncs[slot].variables, name);
                        else
                            syncs[slot].variables[$ name] = value;
                    }
                }
            }
            break;
        case 13: // Highscore update
            var highscore_id = _buf_read_leb_u64(buf);
            var highscore_user = _buf_read_leb_u64(buf);
            var highscore_score = buffer_read(buf, buffer_f64);
            if !struct_exists(global.__game_highscores, highscore_id) {
                var highscore = _create_highscore();
                highscore.id = highscore_id;
                global.__game_highscores[$ highscore_id] = highscore;
            }
            global.__game_highscores[$ highscore_id].scores[$ highscore_user] = highscore_score;
            break;
        case 14: // Update syncs and variables
            _player_id = _buf_read_leb_u64(buf);
            syncs = _buf_read_syncs(buf);
            variables = _buf_read_struct(buf);
            if struct_exists(global.__players, _player_id) {
                player = global.__players[$ _player_id];
                player.syncs = syncs;
                player.variables = variables;
            }
            break;
        case 15: // Request variable from another player
            _player_id = _buf_read_leb_u64(buf);
            var index = buffer_read(buf, buffer_u16);
            if index >= array_length(global.__callback_other_vari) || global.__callback_other_vari[index] == undefined
                return;
            var callback = global.__callback_other_vari[index];
            name = callback[0];
            var func = callback[1];
            var value = _buf_read_value(buf);
            if struct_exists(global.__players, _player_id) {
                global.__players[$ _player_id].variables[$ name] = value;
                func(_player_id, name, value);
            }
            global.__callback_other_vari[index] = undefined;
            break;
        case 16: // Banned / Kicked from the game
            var action = buffer_read(buf, buffer_u8);
            var reason = _buf_read_string(buf);
            switch action {
                case 0: // Banned
                    var time = _buf_read_i64(buf);
                    if global.__script_banned != undefined
                        global.__script_banned(time, reason);
                    else if global.__script_disconnected != undefined
                        global.__script_disconnected();
                    break;
                case 1: // Kicked
                    if global.__script_kicked != undefined
                        global.__script_kicked(reason);
                    else if global.__script_disconnected != undefined
                        global.__script_disconnected();
                    break;
            }
            _crystal_partial_clear_disconnected();
            break;
        case 17: // Request sync variable of another player
            _player_id = _buf_read_leb_u64(buf);
            index = buffer_read(buf, buffer_u16);
            _iter_missing_data(_player_id);
            if index >= array_length(global.__callback_other_vari) || global.__callback_other_vari[index] == undefined
                return;
            callback = global.__callback_other_vari[index];
            name = callback[0];
            func = callback[1];
            slot = callback[2];
            var remove_vari = buffer_read(buf, buffer_u8);
            buffer_seek(buf, buffer_seek_relative, -1);
            value = _buf_read_value(buf);
            if struct_exists(global.__players, _player_id) {
                var sync = global.__players[$ _player_id].syncs[slot];
                if remove_vari
                    struct_remove(sync.variables, name);
                else
                    sync.variables[$ name] = value;
                func(_player_id, name, value);
            }
            global.__callback_other_vari[index] = undefined;
            break;
        case 18: // Update game server version
            global.__game_version = buffer_read(buf, buffer_f64);
            break;
        case 19: // Add administrator
			if _buf_read_bool(buf) {
	            var admin = _create_administrator();
	            admin.id = _buf_read_leb_u64(buf);
	            admin.can_kick = _buf_read_bool(buf);
	            admin.can_ban = _buf_read_bool(buf);
	            admin.can_unban = _buf_read_bool(buf);
	            global.__game_adminstrators[$ admin.id] = admin;
			} else {
				var admin = _buf_read_leb_u64(buf);
				struct_remove(global.__game_adminstrators, admin);
			}
            break;
		case 20: // Forced disconnection
			global.__is_connected = false;
            if global.__call_disconnected {
                if global.__script_disconnected != undefined
                    global.__script_disconnected();
                global.__call_disconnected = false;
            }
            _crystal_clear_disconnected();
			break;
        case 21: // Player ini write
            size = _buf_read_leb_u64(buf);
            repeat size {
                name = _buf_read_string(buf);
                remove_vari = buffer_read(buf, buffer_u8) == 0xff;
                buffer_seek(buf, buffer_seek_relative, -1);
                var vari = _buf_read_value(buf);
                if remove_vari
                    struct_remove(global.__player_save, name);
                else
                    global.__player_save[$ name] = vari;
            }
            break;
    }
}

function _get_room() {
    if global.__script_room != undefined
        return string(global.__script_room());
    return string(room);
}

function _uri_encode(str) {
    var encoded = "";
    for (var i = 1; i <= string_length(str); i++) {
        var char = string_char_at(str, i);
        var convert = true;
        if string_byte_length(char) == 1 {
            var byte = string_byte_at(char, 1);
            // This also doesn't convert the "reserved characters"
            if (byte >= 0x41 && byte <= 0x5a) || (byte >= 0x61 && byte <= 0x7a) || (byte >= 0x30 && byte <= 0x39) ||
                byte == 0x2d || byte == 0x5f || byte == 0x2e || byte == 0x21 || byte == 0x7e || byte == 0x2a ||
                byte == 0x27 || byte == 0x28 || byte == 0x29 || byte == 0x3b || byte == 0x2f || byte == 0x3f ||
                byte == 0x3a || byte == 0x40 || byte == 0x26 || byte == 0x3d || byte == 0x2b || byte == 0x24 ||
                byte == 0x2c || byte == 0x23 {
                    convert = false;
            }
        }
        if convert {
            var hexes = [];
            for (var ii = 1; ii <= string_byte_length(char); ii++) {
                var hex = "";
                var num = string_byte_at(char, ii);
                while num > 0 {
                    hex = string_char_at("0123456789abcdef", (num & 0xf) + 1) + hex;
                    num = num >> 4;
                }
                array_push(hexes, hex);
            }
            for (var ii = 0; ii < array_length(hexes); ii++) {
                var hex = hexes[ii];
                encoded += "%" + (string_length(hex) == 1 ? "0" + hex : hex);
            }
        } else
            encoded += char;
    }
    return encoded;
}

function _uri_decode(str) {
    var decoded = "";
    var is_decoding = false;
    var index_decoding = 0;
    var bytes = "";
    var decoding_data = "";
    var expected_decoder = 0;
    for (var i = 1; i <= string_length(str); i++) {
        var char = string_char_at(str, i);
        if is_decoding {
            decoding_data += char;
            index_decoding++;
            if index_decoding == 2 {
                var num = 0;
                for (var ii = 1; ii <= string_length(decoding_data); ii++)
                    num = num << 4 | (string_pos(string_lower(string_char_at(decoding_data, ii)), "0123456789abcdef") - 1);
                if num & 0b11000000 == 0b11000000 {
                    if num & 0b11110000 == 0b11110000
                        expected_decoder = 3;
                    else if num & 0b11100000 == 0b11100000
                        expected_decoder = 2;
                    else
                        expected_decoder = 1;
                    bytes += string_set_byte_at(" ", 1, num);
                } else if num & 0b10000000 == 0b10000000 {
                    if expected_decoder > 0 {
                        expected_decoder--;
                        if expected_decoder == 0 {
                            decoded += bytes + string_set_byte_at(" ", 1, num);
                            bytes = "";
                        } else
                            bytes += string_set_byte_at(" ", 1, num);
                    } else
                        show_error("Unexpected continuation byte", false);
                } else if expected_decoder > 0
                    show_error("Expected continuation byte", false);
                else
                    decoded += string_set_byte_at(" ", 1, num);
                is_decoding = false;
                index_decoding = 0;
                decoding_data = "";
            }
        } else if char == "%"
            is_decoding = true;
        else
            decoded += char;
    }
    return decoded;
}

function _get_unix_time() {
    var timezone = date_get_timezone();
    date_set_timezone(timezone_utc);
    var timestamp = date_second_span(date_create_datetime(1970, 1, 1, 0, 0, 0), date_current_datetime());
    date_set_timezone(timezone);
    return timestamp;
}

function _get_value_valid(val) {
    switch typeof(val) {
        case "undefined":
        case "null":
            return undefined;
        case "bool":
            return val;
        case "int32":
        case "int64":
            return val;
        case "number":
            return val;
        case "string":
            return val;
        case "array":
            return val;
        case "struct":
            return val;
        case "ref":
            if buffer_exists(val) {
                return val;
            } else if ds_exists(val, ds_type_list) {
                return val;
            } else if ds_exists(val, ds_type_map) {
                return val;
            } else {
                show_debug_message("Unknown value type: {0}, defaulting to undefined", val);
                return undefined;
            }
        default:
            show_debug_message("Unknown value type: {0}, defaulting to undefined", val);
            return undefined;
    }
}

function _get_save_key(file, section, key) {
    return (string_length(file) != 0 ? (_uri_encode(file) + ">") : "") + _uri_encode(section) + ">" + _uri_encode(key);
}

function _check_players_queue(pid) {
    if !struct_exists(global.__players_queue, pid)
        global.__players_queue[$ pid] = _create_playerqueue();
}

function crystal_connect() {
    if _crystal_verify_init() {
        if global.__is_connected {
            return;
        }
        if global.__socket != undefined
            network_destroy(global.__socket);
        if os_browser == browser_not_a_browser
            global.__socket = network_create_socket(network_socket_tcp);
        else
            global.__socket = network_create_socket(network_socket_ws);
        var port = 16562;
        if os_browser == browser_not_a_browser {
            port = 16561;
        }
        for (var i = 0; i < array_length(global.__buffered_data); i++) {
            buffer_delete(global.__buffered_data[i]);
        }
        array_delete(global.__buffered_data, 0, array_length(global.__buffered_data));
        global.__call_disconnected = true;
        global.__is_connecting = true;
        global.__handshake_completed = false;
        var __a = "server";
        var __b = ".";
        global.__async_network_id = network_connect_raw_async(global.__socket, __a + __b + "crystal-server.co", port);
        if global.__async_network_id < 0 {
            global.__is_connected = false;
            global.__is_connecting = false;
            if global.__call_disconnected {
                if global.__script_disconnected != undefined
                    global.__script_disconnected();
                global.__call_disconnected = false;
            }
            _crystal_clear_disconnected();
        }
        var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 0);
        buffer_write(b, buffer_u64, 0xf2b9c7a65420e78);
        buffer_write(b, buffer_u64, 0xb90a8cda60435ab);
        buffer_write(b, buffer_u64, 0xa89c0a6b789adb0);
        buffer_write(b, buffer_u64, 0x19840a684219abd);
        buffer_write(b, buffer_u32, 0);
        var device_info = os_get_info();
        _buf_write_string(b, device_info[? "udid"]);
        ds_map_destroy(device_info);
        _buf_write_string(b, global.__game_id);
        buffer_write(b, buffer_f64, global.__version);
        _buf_write_string(b, global.__session);
        _buf_send(b);
    }
}

function crystal_disconnect() {
    network_destroy(global.__socket);
    if os_browser == browser_not_a_browser
        global.__socket = network_create_socket(network_socket_tcp);
    else
        global.__socket = network_create_socket(network_socket_ws);
    global.__is_connected = false;
    global.__is_connecting = false;
    _crystal_clear_disconnected();
}

function crystal_info_is_connecting() {
    return global.__is_connecting || (!global.__handshake_completed && global.__is_connected);
}

function crystal_script_set_room(script) {
    global.__script_room = script;
}

function crystal_script_set_p2p(script) {
    global.__script_p2p = script;
}

function crystal_script_set_register(script) {
    global.__script_register = script;
}

function crystal_script_set_login(script) {
    global.__script_login = script;
}

function crystal_script_set_banned(script) {
    global.__script_banned = script;
}

function crystal_script_set_kicked(script) {
    global.__script_kicked = script;
}

function crystal_script_set_disconnected(script) {
    global.__script_disconnected = script;
}

function crystal_script_set_login_token(script) {
    global.__script_login_token = script;
}

function crystal_login(username, password) {
    var b = buffer_create(0, buffer_grow, 1);
    buffer_write(b, buffer_u8, 1);
    _buf_write_string(b, username);
    _buf_write_bool(b, false);
    _buf_write_string(b, password);
    _buf_write_string(b, global.__game_token);
    _buf_write_struct(b, global.__variables);
    _buf_write_syncs(b, global.__syncs);
    var r = _get_room();
    _buf_write_string(b, r);
    global.__current_room = r;
    _buf_send(b);
}

function crystal_login_with_token(username, token) {
    var b = buffer_create(0, buffer_grow, 1);
    buffer_write(b, buffer_u8, 1);
    _buf_write_string(b, username);
    _buf_write_bool(b, true);
    _buf_write_string(b, token);
    _buf_write_string(b, global.__game_token);
    _buf_write_struct(b, global.__variables);
    _buf_write_syncs(b, global.__syncs);
    var r = _get_room();
    _buf_write_string(b, r);
    global.__current_room = r;
    _buf_send(b);
}

function crystal_set_game_token(token) {
    global.__game_token = token;
}

function crystal_register(username, email, password, repeat_password) {
    var b = buffer_create(0, buffer_grow, 1);
    buffer_write(b, buffer_u8, 2);
    _buf_write_string(b, username);
    _buf_write_string(b, email);
    _buf_write_string(b, password);
    _buf_write_string(b, repeat_password);
    _buf_send(b);
}

function crystal_info_get_ping() {
    return global.__ping;
}

function crystal_info_is_connected() {
    return global.__is_connected;
}

function crystal_info_is_loggedin() {
    return global.__is_loggedin && global.__is_connected;
}

function crystal_self_get_id() {
    return global.__player_id;
}

function crystal_self_get_name() {
    return global.__player_name;
}

function crystal_self_set(name, value) {
    value = _get_value_valid(value);
    if struct_exists(global.__variables, name) {
        if global.__variables[$ name] == value
            return;
    }
    global.__variables[$ name] = value;
    var u = _create_updatevari();
    u.name = name;
    u.value = value;
    array_push(global.__update_vari, u);
}

function crystal_self_remove(name) {
    if !struct_exists(global.__variables, name)
        return;
    struct_remove(global.__variables, name);
    var u = _create_updatevari();
    u.removed = true;
    u.name = name;
    array_push(global.__update_vari, u);
}

function crystal_other_iter(script) {
    var player_keys = struct_get_names(global.__players);
    for (var i = 0; i < array_length(player_keys); i++) {
        if struct_exists(global.__players, player_keys[i])
            script(real(player_keys[i]), global.__players[$ player_keys[i]].name);
    }
}

function crystal_other_count() {
    return struct_names_count(global.__players);
}

function crystal_other_get_room(pid) {
    if struct_exists(global.__players, pid)
        return global.__players[$ pid].room;
    return "";
}

function crystal_other_get_name(pid) {
    if struct_exists(global.__players, pid)
        return global.__players[$ pid].name;
    return "";
}

function crystal_other_get_pid(name) {
	var players = struct_get_names(global.__players);
	for (var i = 0; i < array_length(players); i++) {
		var pid = players[i];
		var player = global.__players[$ pid];
		if string_lower(player.name) == string_lower(name) {
			return pid;
		}
	}
	return -1;
}

function crystal_other_get(pid, name, default_value = undefined) {
    if struct_exists(global.__players, pid) {
        var vari = global.__players[$ pid].variables;
        if struct_exists(vari, name)
            return vari[$ name];
    }
    return default_value;
}

function crystal_other_has(pid, name) {
    if struct_exists(global.__players, pid)
        return struct_exists(global.__players[$ pid].variables, name);
    return false;
}

function crystal_other_request(pid, name, callback = undefined) {
    callback ??= function() {};
    if struct_exists(global.__players, pid) || pid < 0 {
        var index = -1;
        for (var i = 0; i < array_length(global.__callback_other_vari); i++) {
            if global.__callback_other_vari[i] == undefined {
                index = i;
                break;
            }
        }
        if index == -1 {
            index = array_length(global.__callback_other_vari);
            array_push(global.__callback_other_vari, undefined);
        }
        global.__callback_other_vari[index] = [name, callback];
        var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 3);
        _buf_write_bool(b, pid >= 0);
        if pid >= 0
            _buf_write_leb_u64(b, pid);
        _buf_write_string(b, name);
        buffer_write(b, buffer_u16, index);
        _buf_send(b);
    }
}

function crystal_p2p(p2pcode_or_pid, message_id, data = []) {
    var b = buffer_create(0, buffer_grow, 1);
    buffer_write(b, buffer_u8, 4);
    switch p2pcode_or_pid {
        case P2PCode.AllGame:
            buffer_write(b, buffer_u8, 1);
            break;
        case P2PCode.CurrentSession:
            buffer_write(b, buffer_u8, 2);
            break;
        case P2PCode.CurrentRoom:
            buffer_write(b, buffer_u8, 3);
            break;
        default:
            buffer_write(b, buffer_u8, 0);
            _buf_write_leb_u64(b, p2pcode_or_pid);
            break;
    }
    buffer_write(b, buffer_s16, message_id);
    _buf_write_array(b, data);
    _buf_send(b);
}

function crystal_set_version(version) {
    global.__version = version;
    if global.__is_connected {
        var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 5);
        buffer_write(b, buffer_f64, version);
        _buf_send(b);
    }
}

function crystal_get_server_version() {
    return global.__game_version;
}

function crystal_set_session(session) {
    global.__session = session;
    if global.__is_connected {
        var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 6);
        _buf_write_string(b, session);
        _buf_send(b);
    }
}

function crystal_get_session() {
    return global.__session;
}

function crystal_playerini_current() {
    return global.__player_open_save;
}

function crystal_playerini_open(file = "") {
    global.__player_open_save = file;
}

function crystal_playerini_close() {
    crystal_playerini_open();
}

function crystal_playerini_exists(section, key) {
    return struct_exists(global.__player_save, _get_save_key(global.__player_open_save, section, key));
}

function crystal_playerini_read(section, key, default_value = undefined) {
    if crystal_playerini_exists(section, key)
        return global.__player_save[$ _get_save_key(global.__player_open_save, section, key)];
    return default_value;
}

function crystal_playerini_write(section, key, value) {
    value = _get_value_valid(value);
    if crystal_playerini_exists(section, key) {
        if crystal_playerini_read(section, key) == value
            return;
    }
    global.__player_save[$ _get_save_key(global.__player_open_save, section, key)] = value;
    var u = _create_updatevari();
    u.name = _get_save_key(global.__player_open_save, section, key);
    u.value = value;
    array_push(global.__update_playerini, u);
}

function crystal_playerini_remove(section, key) {
    if crystal_playerini_exists(section, key) {
        struct_remove(global.__player_save, _get_save_key(global.__player_open_save, section, key));
        var u = _create_updatevari();
        u.removed = true;
        u.name = _get_save_key(global.__player_open_save, section, key);
        array_push(global.__update_playerini, u);
    }
}

function crystal_gameini_current() {
    return global.__game_open_save;
}

function crystal_gameini_open(file = "") {
    global.__game_open_save = file;
}

function crystal_gameini_close() {
    crystal_gameini_open();
}

function crystal_gameini_exists(section, key) {
    return struct_exists(global.__game_save, _get_save_key(global.__game_open_save, section, key));
}

function crystal_gameini_read(section, key, default_value = undefined) {
    if crystal_playerini_exists(section, key)
        return global.__game_save[$ _get_save_key(global.__game_open_save, section, key)];
    return default_value;
}

function crystal_gameini_write(section, key, value) {
    value = _get_value_valid(value);
    if crystal_gameini_exists(section, key) {
        if crystal_gameini_read(section, key) == value
            return;
    }
    global.__game_save[$ _get_save_key(global.__game_open_save, section, key)] = value;
    var u = _create_updatevari();
    u.name = _get_save_key(global.__game_open_save, section, key);
    u.value = value;
    array_push(global.__update_gameini, u);
}

function crystal_gameini_remove(section, key) {
    if crystal_gameini_exists(section, key) {
        struct_remove(global.__game_save, _get_save_key(global.__game_open_save, section, key));
        var u = _create_updatevari();
        u.removed = true;
        u.name = _get_save_key(global.__game_open_save, section, key);
        array_push(global.__update_gameini, u);
    }
}

function crystal_achievement_exists(aid) {
    return struct_exists(global.__game_achievements, aid);
}

function crystal_achievement_get_name(aid) {
    if crystal_achievement_exists(aid)
        return global.__game_achievements[$ aid].name;
    return "";
}

function crystal_achievement_get_description(aid) {
    if crystal_achievement_exists(aid)
        return global.__game_achievements[$ aid].description;
    return "";
}

function crystal_achievement_is_reached(aid) {
    if crystal_achievement_exists(aid)
        return struct_exists(global.__game_achievements[$ aid].players, crystal_self_get_id());
    return false;
}

function crystal_achievement_reach(aid) {
    if crystal_achievement_exists(aid) && !crystal_achievement_is_reached(aid) {
        var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 14);
        _buf_write_leb_u64(b, aid);
        _buf_send(b);
        global.__game_achievements[$ aid].players[$ crystal_self_get_id()] = floor(_get_unix_time());
    }
}

function crystal_achievement_get(aid) {
    if crystal_achievement_is_reached(aid)
        return global.__game_achievements[$ aid].players[$ crystal_self_get_id()];
    return 0;
}

function crystal_highscore_exists(hid) {
    return struct_exists(global.__game_highscores, hid);
}

function crystal_highscore_get_name(hid) {
    if crystal_highscore_exists(hid)
        return global.__game_highscores[$ hid].name;
    return "";
}

function crystal_highscore_get(hid) {
    if crystal_highscore_exists(hid) {
        var highscore = global.__game_highscores[$ hid];
        if struct_exists(highscore.scores, crystal_self_get_id())
            return highscore.scores[$ crystal_self_get_id()];
    }
    return 0;
}

function crystal_highscore_set(hid, score) {
    if crystal_highscore_exists(hid) {
        var highscore = global.__game_highscores[$ hid];
        if struct_exists(highscore.scores, crystal_self_get_id()) {
            if highscore.scores[$ crystal_self_get_id()] == score
                return;
        }
        highscore.scores[$ crystal_self_get_id()] = score;
        var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 15);
        _buf_write_leb_u64(b, hid);
        buffer_write(b, buffer_f64, score);
        _buf_send(b);
    }
}

function crystal_sync_new(sync, kind) {
    var index = -1;
    for (var i = 0; i < array_length(global.__syncs); i++) {
        if global.__syncs[i] == undefined {
            index = i;
            break;
        }
    }
    if index == -1 {
        index = array_length(global.__syncs);
        array_push(global.__syncs, undefined);
    }
    var ns = _create_sync();
    ns.kind = kind;
    ns.type = sync;
    global.__syncs[index] = ns;
    
    var nsq = _create_newsyncqueue();
    nsq.slot = index;
    nsq.kind = kind;
    nsq.type = sync;
    array_push(global.__new_sync_queue, nsq);
    
    return index;
}

function crystal_sync_set(slot, name, value) {
    value = _get_value_valid(value);
    if slot >= array_length(global.__syncs) || slot < 0
        return;
    if global.__syncs[slot] != undefined {
        var sync = global.__syncs[slot];
        if struct_exists(sync.variables, name) {
            if sync.variables[$ name] == value
                return;
        }
        sync.variables[$ name] = value;
        if !array_contains(sync.to_sync, name)
            array_push(sync.to_sync, name);
    }
}

function crystal_sync_remove(slot, name) {
    if slot >= array_length(global.__syncs) || slot < 0
        return;
    if global.__syncs[slot] != undefined {
        var sync = global.__syncs[slot];
        if !struct_exists(sync.variables, name)
            return;
        struct_remove(sync.variables, name);
        if !array_contains(sync.to_sync, name)
            array_push(sync.to_sync, name);
    }
}

function crystal_sync_get(pid, sync_slot, name, default_value = undefined) {
    if struct_exists(global.__players, pid) {
        var syncs = global.__players[$ pid].syncs;
        if sync_slot < array_length(syncs) && sync_slot >= 0 {
            if syncs[sync_slot] != undefined {
                var vari = syncs[sync_slot].variables;
                if struct_exists(vari, name)
                    return vari[$ name];
            }
        }
    }
    return default_value;
}

function crystal_sync_has(pid, sync_slot, name) {
    if struct_exists(global.__players, pid) {
        var syncs = global.__players[$ pid].syncs;
        if sync_slot < array_length(syncs) && sync_slot >= 0 {
            if syncs[sync_slot] != undefined {
                return struct_exists(syncs[sync_slot].variables, name);
            }
        }
    }
    return false;
}

function crystal_sync_destroy(slot) {
    if slot < array_length(global.__syncs) && slot >= 0 && global.__syncs[slot] != undefined {
        global.__syncs[slot] = undefined;
        array_push(global.__syncs_remove, slot);
    }
}

function crystal_sync_iter(script) {
    var r = _get_room();
    var player_keys = struct_get_names(global.__players);
    for (var i = 0; i < array_length(player_keys); i++) {
        var player = global.__players[$ player_keys[i]];
        for (var ii = 0; ii < array_length(player.syncs); ii++) {
            if player.syncs[ii] != undefined {
                var sync = player.syncs[ii];
                if player.room != r && sync.event != SyncEvent.End
                    continue;
                var s = _create_synciter();
                s.event = sync.type != CreateSync.Once ? sync.event : SyncEvent.Once;
                s.kind = sync.kind;
                s.variables = sync.variables;
                s.player_id = real(player_keys[i]);
                s.player_name = player.name;
                s.slot = ii;
                script(s);
                if sync.event == SyncEvent.New
                    sync.event = SyncEvent.Step;
            }
        }
    }
}

function crystal_sync_request(pid, slot, variable_name, callback = undefined) {
    callback ??= function() {};
    if struct_exists(global.__players, pid) {
        var player = global.__players[$ pid];
        if slot < array_length(player.syncs) && slot >= 0 && player.syncs[slot] != undefined {
            var index = -1;
            for (var i = 0; i < array_length(global.__callback_other_vari); i++) {
                if global.__callback_other_vari[i] == undefined {
                    index = i;
                    break;
                }
            }
            if index == -1 {
                index = array_length(global.__callback_other_vari);
                array_push(global.__callback_other_vari, undefined);
            }
            global.__callback_other_vari[index] = [variable_name, callback, slot];
            var b = buffer_create(0, buffer_grow, 1);
			buffer_write(b, buffer_u8, 17);
            _buf_write_leb_u64(b, pid);
            _buf_write_string(b, variable_name);
            buffer_write(b, buffer_u16, index);
            buffer_write(b, buffer_u16, slot);
            _buf_send(b);
        }
    }
}

function crystal_self_is_admin() {
    return struct_exists(global.__game_adminstrators, crystal_self_get_id());
}

function crystal_self_get_admin() {
    if crystal_self_is_admin()
        return global.__game_adminstrators[$ crystal_self_get_id()];
    return undefined;
}

function crystal_other_is_admin(pid) {
    return struct_exists(global.__game_adminstrators, pid);
}

function crystal_other_get_admin(pid) {
    if crystal_other_is_admin(pid)
        return global.__game_adminstrators[$ pid];
    return undefined;
}

function crystal_admin_kick(pid, reason = "") {
    if pid == crystal_self_get_id() || crystal_self_get_admin().can_kick {
        var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 16);
        buffer_write(b, buffer_u8, 2);
        _buf_write_leb_u64(b, pid);
        _buf_write_string(b, reason);
        _buf_send(b);
        return true;
    }
    return false;
}

function crystal_admin_ban(pid, unix_unban_time, reason = "") {
    if pid == crystal_self_get_id() || crystal_self_get_admin().can_ban {
        var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 16);
        buffer_write(b, buffer_u8, 0);
        _buf_write_leb_u64(b, pid);
        _buf_write_string(b, reason);
        _buf_write_i64(b, unix_unban_time);
        _buf_send(b);
        return true;
    }
    return false;
}

function crystal_admin_unban(pid) {
    if crystal_self_get_admin().can_unban {
        var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 16);
        buffer_write(b, buffer_u8, 1);
        _buf_write_leb_u64(b, pid);
        _buf_send(b);
        return true;
    }
    return false;
}

function crystal_use_compression_zlib(use) {
    global.__compression = use ? CompressionType.Zlib : CompressionType.None;
}

function crystal_logout() {
	if crystal_info_is_loggedin() {
		var b = buffer_create(0, buffer_grow, 1);
        buffer_write(b, buffer_u8, 18);
        _buf_send(b);
		_crystal_partial_clear_disconnected();
		return true;
	}
	return false;
}
