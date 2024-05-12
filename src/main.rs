use colored::{CustomColor, Colorize};
use std::time::Instant;
use console::Term;

/// whether to show debug info in the game
const DEBUG_INFO: bool = true;

/// specifies a vector that expresses the direction of the velocity of the player.
/// Gets multiplied by delta time and added to position every frame
static mut MOVEMENT_DIRECTION: Vector2 = Vector2::const_new();

/// Position of top left corner of camera's view.
static mut CAMERA_POSITION: Vector2 = Vector2::const_new();
static mut PLAYER_POSITION: Vector2 = Vector2::const_new();

const WORLD_SIZE_X: usize = 50;
const WORLD_SIZE_Y: usize = 50;
const WORLD_SAVE_PATH: &str = "world.save";

const SCREEN_SIZE_X: usize = 40;
const SCREEN_SIZE_Y: usize = 40;

/// We need a minimum frame time because if we draw too fast to terminal the "image"
/// will appear torn or incomplete <br>
/// 10 milliseconds (100fps (actually 90fps)) appears to be the bare minimum,
/// at least on my laptop with Konsole
const MIN_FRAME_TIME: u128 = 50_000; // in microseconds

/// static variable that holds time frame time
static mut DELTA_TIME: u128 = 100_000;

fn main() {
  
  // main thread, the game thread. mostly draws to screen and calculates delta time
  // MARK: Game thread
  let game_thread = std::thread::spawn(move || {
    // INITIATE the screen
    let mut screen: Screen = Screen { pixels: Vec::new(), size_x: SCREEN_SIZE_X, size_y: SCREEN_SIZE_Y };

    // LOAD the world
    let unrendered_world = load_world(WORLD_SAVE_PATH);

    // initialise frame start and end times to be used to calculate delta_time (frame time)
    let mut frame_start_time: Instant = Instant::now();
    let mut frame_end_time: Instant = Instant::now();
    
    loop /* game loop */ {
      // calculate frame time
      unsafe {DELTA_TIME = (frame_end_time - frame_start_time).as_micros();}
      // set frame start time for this frame
      frame_start_time = Instant::now();

      // clear the screen
      screen.init();

      // RENDER the world
      let mut rendered_world: Vec<Square> = Vec::new();
      // as of now, basic rendering. No shading applied. Uses the default `new()` function to generate colours and ascii art.
      for unrendered_square in unrendered_world.clone() {
        rendered_world.push(Square::new(unrendered_square));
      }

      screen = render_world(screen, rendered_world);

      // PLAYER
      unsafe {
        // update player pos
        CAMERA_POSITION.x += MOVEMENT_DIRECTION.x;
        CAMERA_POSITION.y += MOVEMENT_DIRECTION.y;
        // draw player
        let player_x: i16 = CAMERA_POSITION.x;
        let player_y: i16 = CAMERA_POSITION.y;
        screen.pixels[player_y as usize][player_x as usize] = Square::new(SquareType::Player);
      }

      // show debug info
      if DEBUG_INFO {
        unsafe {
          println!("frame time : {:?}μs",  DELTA_TIME              );
          println!("frame rate : {:?}fps", 1_000_000/(DELTA_TIME+1));
          println!("camera pos : {:?}",    CAMERA_POSITION         );
          println!("velocity   : {:?}",    MOVEMENT_DIRECTION      );
        }
      }

      // draw main screen to terminal
      screen.draw();

      // sleep until we reach min frame time
      if frame_start_time.elapsed().as_millis() < MIN_FRAME_TIME {
        std::thread::sleep(std::time::Duration::from_micros((MIN_FRAME_TIME - frame_start_time.elapsed().as_millis()) as u64));
      }
      clearscreen::clear().expect("failed to clear screen");
      frame_end_time = Instant::now();
    }
  });
    
  // MARK: Input thread
  let input_thread = std::thread::spawn(|| {
    
    loop {
      let key = Term::buffered_stdout().read_key().expect("idc");
      
      if DEBUG_INFO {
        println!("PRESSED: {:?}", key);
      }
      let mut horizontal_movement: i16 = 0;
      let mut vertical_movement: i16 = 0;
      // unsafe because static mutation (why is this unsafe?)
      match key {

        console::Key::Char('a') => {
          horizontal_movement = -1;
        }
        console::Key::Char('d') => {
          horizontal_movement = 1;
        }
        console::Key::Char('w') => {
          vertical_movement = -1;
        }
        console::Key::Char('s') => {
          vertical_movement = 1;
        }
        _ => {
          horizontal_movement = 0;
        }
      }
      unsafe { MOVEMENT_DIRECTION.x = horizontal_movement};
      unsafe { MOVEMENT_DIRECTION.y = vertical_movement};

    }
  });

  let reset_thread = std::thread::spawn(|| {
    loop {
      // basically "let char = stdout.read_char().expect("ass");" won't let the program move on
      // until a key is pressed so we need to reset the velocity x component to zero, otherwise
      // it stays 1 or -1 or whatever it was.

      // so we solve the issue caveman style by periodically setting x movement direction back to 0
      unsafe {std::thread::sleep(std::time::Duration::from_micros(DELTA_TIME as u64 * 5));}
      unsafe {MOVEMENT_DIRECTION.x = 0;}
      unsafe {MOVEMENT_DIRECTION.y = 0;}
    }
  });


  let _ = game_thread.join();
  let _ = input_thread.join();
  let _ = reset_thread.join();
}

// MARK: World load/render
/// Loads the world into a vector of SquareTypes.
fn load_world(save_path: &str) -> Vec<SquareType> {
  let contents: String = std::fs::read_to_string(save_path)
    .expect("Should have been able to read the file, or not idk");

  let mut square_type_list: Vec<SquareType> = Vec::new();

  for square_type in contents.split_whitespace() {
    square_type_list.push(match square_type {
      "0" => SquareType::Air,
      "1" => SquareType::Grass,
      _ => panic!()
    });
  }
  return square_type_list;
}

fn render_world(mut screen: Screen, square_list:Vec<Square>) -> Screen {
  // for x in player_pos_x -> +windowsize
  // for y in player_pos_< -> +windowsize
  // block_at_pos = blocklist[x * y]
  // screen[x][y] = block_at_pos
  unsafe {
    for x in CAMERA_POSITION.x..(SCREEN_SIZE_X as i16 + CAMERA_POSITION.x as i16) {
      for y in CAMERA_POSITION.y..(SCREEN_SIZE_Y as i16 + CAMERA_POSITION.y as i16) {
        if y < WORLD_SIZE_Y as i16 && x < WORLD_SIZE_X as i16 {
          let square_to_draw = square_list[((x)+(y * WORLD_SIZE_Y as i16)) as usize].clone();
          screen.pixels[(y - CAMERA_POSITION.y) as usize][(x - CAMERA_POSITION.x) as usize] = square_to_draw;
        }
      }
    }
  }

  return screen;
}

// MARK: Structs
#[derive(Debug,Clone,Copy)]
struct Vector2 {
  x: i16,
  y: i16,
}
impl Vector2 {
  /// Initialises a Vector2 with (0;0)
  fn new() -> Vector2 {
    return Vector2 {
      x: 0,
      y: 0,
    }
  }
  /// Initialises a Vector2 with (0;0), but the function is constant
  const fn const_new() -> Vector2 {
    return Vector2 {
      x: 0,
      y: 0,
    }
  }
}

/// This struct represents a square in the world. (Minecraft's equivalent of Blocks).
/// It contains its type, color and the two ascii characters used to draw it in the terminal.
/// The aforementioned two values are generated using `Square::new(SquareType::<square type>)`.
/// ```rust
/// // example -- set coordinates (0; 2) to a grass block on the Screen
/// screen.pixels[0][2] = Square::new(SquareType::Grass);
/// ```
#[derive(Debug, Clone, PartialEq)]
struct Square {
  squaretype: SquareType,
  color: CustomColor,
  ascii: String, // maybe should be str idk
}
/// An enum that defines all the squares in the game.
#[derive(Debug, Clone, PartialEq)]
enum SquareType {
  Air,
  Grass,
  Player,
}
impl Square {
  /// Generates a new Square struct using a SquareType as parameter.
  fn new(squaretype: SquareType) -> Square {

    let color: CustomColor = match squaretype {
      SquareType::Air => CustomColor { r: 0, g: 0, b: 0 },
      SquareType::Grass => CustomColor { r: 0, g: 255, b: 0 },
      SquareType::Player => CustomColor { r: 255, g: 255, b: 255 },
    };

    let ascii: String = match squaretype {
      SquareType::Air => String::from("AA"),
      SquareType::Grass => String::from("GG"),
      SquareType::Player => String::from("PP"),
    };

    return Square {
      squaretype,
      color,
      ascii,
    }
  }
}

/// The screen struct represents the screen. It is a Vector of Vectors of Pixels. The index of the outer
/// vector represents the Y coordinate, and the index of the inner vector the X coordinate.
/// ```
/// + ------- X
/// |
/// |
/// |
/// '
/// Y
/// ```
#[derive(Debug, Clone, PartialEq)]
struct Screen {
  pixels: Vec<Vec<Square>>,
  size_x: usize,
  size_y: usize,
}
impl Screen {
  /// Initiates the screen with blank Squares. (air squares)
  fn init(&mut self) {
    let mut screen: Vec<Vec<Square>> = Vec::new();
    for _ in 0..self.size_y {
      let mut y_row: Vec<Square> = Vec::new();
      for _ in 0..self.size_x {
        y_row.push(Square::new(SquareType::Air));
      }
      screen.push(y_row);
    }
    self.pixels = screen;
  }
  /// Prints the screen to terminal.
  fn draw(&self) {
    for column in 0..self.size_y {
      for pixel in 0..self.size_x {
        let square: Square = self.pixels[column][pixel].clone();
        let color = square.color;
        let pixel_char = square.ascii;
        print!("{}", pixel_char.clone().custom_color(color));
      }
      println!();
    }
  }
}
