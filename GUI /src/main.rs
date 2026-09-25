use tjack::{Color::*, Game, PieceRepresentation, PieceType::*, Ply, Position};
use nannou::{lyon::geom::euclid::rect, prelude::{BLACK, bevy_ecs::error::panic, bevy_render::mesh::Polyline2dMeshBuilder, *}};

// Credit to https://commons.wikimedia.org/wiki/Category:PNG_chess_pieces/Standard_transparent for chess piece assents.


fn main() {
    nannou::app(model)
        .update(update)
        .simple_window(view)
        .run();
}

struct Model {
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


fn model(_app: &App) -> Model {
    Model {
        chesslogic: tjack::Game::new(),
        // texture: _app.asset_server().load("Pieces/Chess_bdt60.png")
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


fn update(_app: &App, _model: &mut Model) {
    if let Some(clicked_square) = klick_square(_app, 70.0) && _model.checkmate != true {
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
    if let Some(clicked_square) = _model.selected_square {
        // Draw selected_square
        draw
            .rect()
            .w(board_size)
            .h(board_size)
            .color(DARK_RED)
            .x_y((clicked_square[0] as f32 * board_size) - board_size * 3.5 , (clicked_square[1] as f32 * board_size) - board_size * 3.5);
    }

    // Drawing pieces on the board.
    draw_pieces(&draw, _model, board_size);

    // draws checkmate to screen if game is in checkmate.
    if _model.checkmate == true {
    //    draw
    //    .rect()
    //    .w(board_size*6.5)
    //    .h(board_size*4.5)
    //    .color(GREY);


        let color = match _model.chesslogic.whose_turn() {
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
                return Some([
                    ((mouse_position[0]+ board_size * 4.0)/board_size) as i32, 
                    ((mouse_position[1] + board_size * 4.0)/board_size) as i32,
                    ])
        } else {
            panic!("ERROR: Pressed out of bounds")
        }
    } 
    None
}

fn draw_pieces(draw: &Draw, _model: &Model, board_size: f32) {

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
        _model: &'a Model, 
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


fn position_from_xy(_model: &mut Model, square_x: i32, square_y: i32) -> Option<Position> {
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

