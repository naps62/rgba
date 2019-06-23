pub static mut VERBOSITY: u8 = 0;

pub fn set_verbosity_level(level: u8) {
  unsafe {
    VERBOSITY = level;
  }
}

#[macro_export]
macro_rules! debug {
  ($level:expr, $pattern:expr $(, $opt:expr)*) => {
      if unsafe { debug::VERBOSITY >= $level } {
        println!($pattern, $($opt),*);
    }
  };
}
