use tjack::{Color::*, Game, PieceRepresentation, PieceType::*, Ply, Position};
use nannou::{image::EncodableLayout, lyon::geom::euclid::rect, prelude::{BLACK, bevy_ecs::{error::panic, message}, bevy_render::mesh::Polyline2dMeshBuilder, *}};
use core::time;
use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream}, os::unix::net::SocketAddr, thread::sleep};
// Credit to https://commons.wikimedia.org/wiki/Category:PNG_chess_pieces/Standard_transparent for chess piece assents.


// #####################
// # Parser Function   #
// #####################

// parses acks. 

// parses one line of movement
fn parse_movement(message: &String) -> (Option<[i32; 2]>, Option<[i32; 2]>, bool, [char; 64]) {
    let msg = "Could not index message";
    // Checks if its a move message or ack
    // Move message.

    // Converts protocal notation to GUI notation.
    let x_pre = message.chars().nth(0).expect(msg) as u8 - 'A' as u8;
    let y_pre = message.chars().nth(1).expect(msg) as u8 - '1' as u8;

    let x_post = message.chars().nth(2).expect(msg) as u8 - 'A' as u8;
    let y_post = message.chars().nth(3).expect(msg) as u8 - '1' as u8;

    // Checks if promotion is happening, Chess lib does not have it -> insta reject
    let doing_promotion = message.chars().nth(4).expect(msg) != '-';

    // parse board part to an array.
    let mut other_board = ['Z'; 64];
    for i in 0..=63 {
        other_board[i] = message.chars().nth(5 + i).expect(msg);
    }


    // Returns the "selected square" and "the movement"
    return (Some([x_pre as i32, y_pre as i32]), Some([x_post as i32, y_post as i32]), doing_promotion, other_board);
}

fn protocal_move_notation(klick: Option<[i32; 2]>) -> String {
    let actual_klick = klick.expect("No klick"); 
    let x = actual_klick[0] as u8 + 'A' as u8;
    let y = actual_klick[1] as u8 + '1' as u8;
    return (x as char).to_string() + &(y as char).to_string()
}

// Compares two boards, arrays of 64 chars
fn compare_boards(board_1: [char; 64], board_2: [char; 64]) -> bool {
    for i in 0..=63 {
        if board_1[i] != board_2[i] {
            return false;
        }
    }
    return true;
}

// Creates a array of chars from matrix.
fn create_array_from_matrix(boardmatrix: [[Option<PieceRepresentation>; 8]; 8]) -> [char; 64] {
    let mut char_array = ['z'; 64];
        
        for y in 0..8 {
            for x in 0..8 {
            let character = match boardmatrix[y][x] {
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: PAWN,
                }) => 'p',
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: PAWN,
                }) => 'P',
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: BISHOP,
                }) => 'b',
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: BISHOP,
                }) => 'B',
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: KING,
                }) => 'k',
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: KING,
                }) => 'K',
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: KNIGHT,
                }) => 'n',
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: KNIGHT,
                }) => 'N',
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: QUEEN,
                }) => 'q',
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: QUEEN,
                }) => 'Q',
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: ROOK,
                }) => 'r',
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: ROOK,
                }) => 'R',
                // If piece cannot be matched set is_piece to false.
                _ => ' ',
            };
        char_array[y*8 + x] = character;
        }
    }
    return char_array;
}


// #####################
// # Network Functions #
// #####################

// Setting upp network, returns the stream and true if listner, false if reciver.
fn network_startup() -> io::Result<(TcpStream, bool)> {
    // Input for checking what connection type is used.
    println!("1 for setting listner.");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("failed to readline");

    if input.trim() == "1".to_string() {
        // Binds a listner to 127.0.0.1:6767
        let listener = TcpListener::bind("127.0.0.1:6767")?;

        let incoming_stream = listener.accept(); 
            println!("Connection successful");
            // Creates a variable to handle ownership
            let (mut stream, _temp): (TcpStream, std::net::SocketAddr) = incoming_stream.unwrap();
            stream.set_nonblocking(true).expect("set_nonblocking call failed");
            //listner_logic(&mut stream);
            return Ok((stream, true));
    } else {
        // Takes address input.
        println!("Address: ");
        let mut addr = String::new();
        io::stdin().read_line(&mut addr).expect("failed to readline");
        // Tries to connect to a listner.
        let stream = TcpStream::connect(addr.trim()).expect("Could not connect");
        stream.set_nonblocking(true).expect("set_nonblocking call failed");
        // Expect connection to be successful.
        //Read_logic(&mut stream);
        return Ok((stream, false));
    }
}


// Logic for reading from the stream, Reads until it cannot read anymore lines then returns a vec with all lines.
fn read_logic(stream: &mut TcpStream) -> Vec<String> {
    // Init return value,
    let mut lines: Vec<String> = Vec::new();
    // Loops over all messages, if no characters can be read exit loop, Reading logic.
    '_outer: loop {
        // Reads stream Logic.
        let mut buf: [u8 ; 1] = [0; 1];
        let mut read_buf: Vec<u8>  = Vec::new();
        // Loops over all chars in a given message.
        '_inner: loop {
            match stream.read(&mut buf) {
                // Matches on if nothing can be read.
                Ok(0) => {
                    // This loops forever, check 
                    eprintln!("Connection lost.");
                    sleep(time::Duration::from_secs(1));
                }
                // If something can be read than read
                Ok(_n) => {
                    // Loops on inner, checks all values until newline.
                        // Checks until newline
                        if String::from_utf8_lossy(&buf) == "\n".to_string(){
                            // When full line has been read print then break.
                            let line = String::from_utf8(read_buf).expect("String could not be converted");
                            lines.push(line);
                            break '_inner;
                        } else {
                            // adds char to read_buf
                            read_buf.push(buf[0]);
                            }
                    }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        break '_outer;
                    }
                _ => {
                    // Matches on Error.
                    eprintln!("Could not read from stream")
                }
            }
        }
    }
    return lines;
}

// #################
// # Main Function #
// #################

fn main() -> std::io::Result<()> {
    nannou::app(model)
       .update(update)
       .simple_window(view)
       .run();
    
    Ok(())
}

// ##################
// # Game Functions #
// ##################

#[derive(Clone)]
struct GUI_model {
    chesslogic: tjack::Game,
    selected_square: Option<[i32; 2]>,
    checkmate: bool,
    possible_plys: Option<Vec<Ply>>,

    // All file paths for the piece textures.
    black_pawn: Handle<Image>,
    white_pawn: Handle<Image>,

    black_bishop: Handle<Image>,
    white_bishop: Handle<Image>,

    black_king: Handle<Image>,
    white_king: Handle<Image>,

    black_knight: Handle<Image>,
    white_knight: Handle<Image>,

    black_queen: Handle<Image>,
    white_queen: Handle<Image>,

    black_rook: Handle<Image>,
    white_rook: Handle<Image>,
}

struct Model {
    network_stream: TcpStream,

    // PlaceHolder, see if switch types is needed.
    player_color: tjack::Color,

    gui_model: GUI_model,
}

// Startup part of nannau.
fn model(_app: &App) -> Model {

    // ##################
    // # Network setup  #
    // ##################

    let (mut stream, is_listner) = network_startup().expect("Could not start network connection");
    let mut color: Option<tjack::Color> = None;

    // If it is the listner
    if is_listner {
        stream.write_all("W\n".as_bytes());
        color = Some(tjack::Color::BLACK);
    } else {
        // If client.
        loop {
            // loops until the first line comes through.
            let read_lines = read_logic(&mut stream);
            if read_lines.len() > 0 {
                let firstline = read_lines[0].clone();
                color = match firstline.as_str() {
                    "W" => Some(tjack::Color::WHITE),
                    "B" => Some(tjack::Color::BLACK),
                    _ => None,
                };
                break;
                }
        } 
    }

    // ##################
    // #   Game setup   #
    // ##################
    Model {
        network_stream: stream,
        // PLACEHOLDER.
        player_color: color.expect("No first color recived"),

        gui_model: GUI_model {
        chesslogic: tjack::Game::new(),
        selected_square: None,
        checkmate: false,
        possible_plys: None,

        black_pawn: _app.asset_server().load("Pieces/Chess_pdt60.png"),
        white_pawn: _app.asset_server().load("Pieces/Chess_plt60.png"),

        black_bishop: _app.asset_server().load("Pieces/Chess_bdt60.png"),
        white_bishop: _app.asset_server().load("Pieces/Chess_blt60.png"),

        black_king: _app.asset_server().load("Pieces/Chess_kdt60.png"),
        white_king: _app.asset_server().load("Pieces/Chess_klt60.png"),

        black_knight: _app.asset_server().load("Pieces/Chess_ndt60.png"),
        white_knight: _app.asset_server().load("Pieces/Chess_nlt60.png"),

        black_queen: _app.asset_server().load("Pieces/Chess_qdt60.png"),
        white_queen: _app.asset_server().load("Pieces/Chess_qlt60.png"),

        black_rook: _app.asset_server().load("Pieces/Chess_rdt60.png"),
        white_rook: _app.asset_server().load("Pieces/Chess_rlt60.png"),
        }
    }
}


fn update(_app: &App, _model: &mut Model) {
    // Check whos turn it is.
    if _model.gui_model.chesslogic.whose_turn() == _model.player_color {
        // If it is the players turn.
        // Normal move logic.
        // Simulates move on board copy.
        if let klick = klick_square(_app, 70.0) {
            let mut board_copy = _model.gui_model.clone();
            core_GUI_move_logic(&mut board_copy, klick);

            // Set variables to board array representation.
            let og_board_array = create_array_from_matrix(_model.gui_model.chesslogic.get_matrix_board_repr());
            let board_copy_array = create_array_from_matrix(board_copy.chesslogic.get_matrix_board_repr());
            
            // Compare boardstates. If move piece -> write to stream, else repeat as normal.
            if compare_boards(og_board_array, 
            board_copy_array) == false {
                // Creates the start of the message.
                let mut message = protocal_move_notation(_model.gui_model.selected_square) + &protocal_move_notation(klick) + &'-'.to_string();
                // Adds all characters of the board.
                for character in board_copy_array {
                    message = message + &character.to_string();
                }
                message = message + "\n";

                println!("{}", message);
                _model.network_stream.write_all(message.as_bytes());
                // Loops until response.
                loop {
                    let read_list = read_logic(&mut _model.network_stream);
                    sleep(time::Duration::from_millis(100));
                    if read_list.len() > 0 {
                        if read_list[0].as_str() == "OK" {
                            _model.gui_model = board_copy.clone();
                        }   else if read_list[0].as_str() == "CHECKMATE" {
                            _model.gui_model = board_copy.clone();
                            _model.gui_model.checkmate = true;
                        } 
                        // Uneccesary for now; else if read_list[0].as_str() == "REJECT" {}
                        break;
                    }
                }
            } else {
                _model.gui_model = board_copy.clone();
            }
        }

    } else {
    // If it is not the players turn.
    // Reads message.
    let message_list = read_logic(&mut _model.network_stream);
    if message_list.len() > 0 {
            let message = &message_list[0];
            println!("{}", message);
            let (pre_position, post_position, has_promotion, boardrep) = parse_movement(message);
            // Copy the boardstate.
            let model_copy = &mut _model.gui_model.clone();
            // Unselects square.
            model_copy.selected_square = None;
            // Simulates selecting a square, then moving a piece.
            core_GUI_move_logic(model_copy, pre_position);
            core_GUI_move_logic(model_copy, post_position);
            
            // Compares the sent boardrep and the boardrep generated. Then sends reply depeending on given context.
            println!("{:?}", boardrep);
            println!("{:?}", create_array_from_matrix(model_copy.chesslogic.get_matrix_board_repr()));
            if boardrep == create_array_from_matrix(model_copy.chesslogic.get_matrix_board_repr()) {
                _model.gui_model = model_copy.clone();
                if _model.gui_model.checkmate {
                    _model.network_stream.write_all("CHECKMATE\n".as_bytes());
                } else {
                    _model.network_stream.write_all("OK\n".as_bytes());
                }
            } else {
                _model.network_stream.write_all("REJECT\n".as_bytes());
            }
        }
    }
}

fn view(app: &App, _model: &Model, _window: Entity) {
    // get canvas to draw on 
    let draw = app.draw();
    // set background color, BLANCHED_ALMOND
    draw.background().color(GREY);

    // Everything drawn to the frame.

    // Drawing the board.
    let board_size = 70.0;

    draw_checker_pattern(&draw, board_size);

    // Draw selected square
    if let Some(clicked_square) = _model.gui_model.selected_square {
        // Draw selected_square
        draw
            .rect()
            .w(board_size)
            .h(board_size)
            .color(DARK_RED)
            .x_y((clicked_square[0] as f32 * board_size) - board_size * 3.5 , (clicked_square[1] as f32 * board_size) - board_size * 3.5);
    }

    // Drawing pieces on the board.
    draw_pieces(&draw, &_model.gui_model, board_size);

    // draws checkmate to screen if game is in checkmate.
    if _model.gui_model.checkmate == true {

        let color = match _model.gui_model.chesslogic.whose_turn() {
            tjack::Color::BLACK => "BLACK",
            tjack::Color::WHITE => "WHITE",
        };

        draw
        .text(&(color.to_owned() + " in Checkmate"))
        .color(BLANCHED_ALMOND)
        .font_size(45)
        .font("Sans");
        return;
    }

}

// draws a checker pattern onto the window.
fn draw_checker_pattern(draw: &Draw, board_size: f32) {
    let mut color = DARK_SLATE_GREY;

    for a in 0..8 {
        // Changes color orientation from different rows.
        if color == BLANCHED_ALMOND {
                color = nannou::prelude::DARK_SLATE_GREY;
            } else {
                color = nannou::prelude::BLANCHED_ALMOND;
            }

        for i in 0..8 {
            if color == nannou::prelude::BLANCHED_ALMOND {
                color = nannou::prelude::DARK_SLATE_GREY;
            } else {
                color = BLANCHED_ALMOND;
            }
            draw.rect()
                .color(color)
                .w(board_size)
                .h(board_size)
                .x_y((i as f32 * board_size) - board_size * 3.5 , (a as f32 * board_size) - board_size * 3.5);
        }
    }
}

// Converts a mouse click to in board notation.
fn klick_square(app: &App, board_size: f32) -> Option<[i32; 2]> {
    let mouse_position = app.mouse();
    // logic to check if mouse is pressed on a square. Marks all squares as x,y : x,y ranges from 0..=8
    if app.mouse_buttons().just_pressed(MouseButton::Left) == true {
        if mouse_position[0] >= -board_size * 4.0 
            && mouse_position[0] <= board_size * 4.0 
            && mouse_position[1] >= -board_size * 4.0 
            && mouse_position[1] <= board_size * 4.0 {
            
                // Selects square. Index: ((mouse_position[0]+ board_size * 4.0)/board_size) as i32, ((mouse_position[1] + board_size * 4.0)/board_size) as i32)
                
                // Debuging : println!("{}, {}", ((mouse_position[0] + board_size * 4.0)/board_size) as i32, ((mouse_position[1] + board_size * 4.0)/board_size) as i32);
                
                return Some([
                    ((mouse_position[0] + board_size * 4.0)/board_size) as i32, 
                    ((mouse_position[1] + board_size * 4.0)/board_size) as i32,
                    ])
        } else {
            panic!("ERROR: Pressed out of bounds")
        }
    } 
    None
}

fn draw_pieces(draw: &Draw, _model: &GUI_model, board_size: f32) {

    let boardmatrix = _model.chesslogic.get_matrix_board_repr();

    for a in 0..8 {
        for i in 0..8 {
            if let Some(texture) = match_matrix_to_texture( _model, boardmatrix, i, a) {

            draw
            .rect()
            .texture(texture)
            .w_h(board_size, board_size)
            .x_y((i as f32 * board_size) - board_size * 3.5 , board_size * 3.5 - (a as f32 * board_size));
            }
        }
    }
}

// Checks what piece is att what offset, if a piece is there return true, else return false.
fn match_matrix_to_texture<'a>(
        _model: &'a GUI_model, 
        boardmatrix: [[Option<PieceRepresentation>; 8]; 8], 
        x: usize, 
        y: usize) -> Option<&'a Handle<Image>> {

            match boardmatrix[y][x] {
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: PAWN,
                }) => Some(&_model.black_pawn),
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: PAWN,
                }) => Some(&_model.white_pawn),
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: BISHOP,
                }) => Some(&_model.black_bishop),
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: BISHOP,
                }) => Some(&_model.white_bishop),
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: KING,
                }) => Some(&_model.black_king),
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: KING,
                }) => Some(&_model.white_king),
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: KNIGHT,
                }) => Some(&_model.black_knight),
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: KNIGHT,
                }) => Some(&_model.white_knight),
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: QUEEN,
                }) => Some(&_model.black_queen),
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: QUEEN,
                }) => Some(&_model.white_queen),
                Some(PieceRepresentation {
                    color: tjack::Color::BLACK,
                    piece_type: ROOK,
                }) => Some(&_model.black_rook),
                Some(PieceRepresentation {
                    color: tjack::Color::WHITE,
                    piece_type: ROOK,
                }) => Some(&_model.white_rook),
                // If piece cannot be matched set is_piece to false.
                _ => None,
            }
}

fn core_GUI_move_logic(_model: &mut GUI_model, movement: Option<[i32; 2]>) {
    // klick_square(_app, 70.0)
    // Logic for selecting square and converting to movement, Core GUI logic
    if let Some(clicked_square) = movement && _model.checkmate != true {
        // Highlighting and movement
        // Selects square 
        if _model.selected_square != None {
            // If a square has been selected
            if let Some([square_x, square_y]) = _model.selected_square.as_ref() {
                if let Some(position) = position_from_xy(_model, *square_x, *square_y) {

                let possible_plys: Vec<Ply> = match _model.chesslogic.find_plies(position) {
                    Some(possible_plys) => possible_plys,
                    None => {
                        println!("None");
                        // reset selected square.
                        _model.selected_square = None;
                        return;
                    }
                };
                // Saves possible plys.
                _model.possible_plys = Some(possible_plys.clone());
                // Perform movement
                for ply in possible_plys {
                    let new_pos = match ply {
                        Ply::Quiet { old_position, new_position} => new_position,
                        Ply::Capture { old_position, new_position, captured_piece } => new_position,
                    };
                    // Converts clicked square to position
                    if let Some(parsed_clicked_square) = position_from_xy(_model, clicked_square[0], clicked_square[1]) {
                        if parsed_clicked_square == new_pos {
                            _model.chesslogic.perform_ply(ply);
                            if _model.chesslogic.is_check() {
                                println!("King in check.");
                                if _model.chesslogic.is_checkmate() {
                                    println!("King in checkmate");
                                    _model.checkmate = true;
                                }
                            }
                        }
                    }
                };
            }
            // Cleanup
                _model.selected_square = None;
                _model.possible_plys = None;
                return;
            };
         // If no square has been selected, select that square.
        } 
    _model.selected_square = Some(clicked_square);
    }
}


// creates a position from x and y cordinates, 0,0 being left upper.
fn position_from_xy(_model: &mut GUI_model, square_x: i32, square_y: i32) -> Option<Position> {
    let file: tjack::File = match tjack::File::try_from(square_x as i8 + 1) {
        Ok(file) => file,
        Err(error) => {
            println!("None");
            // Reset selected square
            _model.selected_square = None;
            return None
        }
    };   
        // Creating Rank 
        let rank: i8 = square_y as i8 + 1;
        // Creating selected_square position
        return Some(tjack::Position::new(file, rank))
}

