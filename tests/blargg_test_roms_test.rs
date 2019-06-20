mod test_helpers;

use test_helpers::integration;

static CPU_INSTR_URL: &str = "http://gbdev.gg8.se/files/roms/blargg-gb-tests/cpu_instrs.zip";

#[test]
fn test() {
  integration::ensure_file_cached(CPU_INSTR_URL, "cpu_instr.zip");
  integration::ensure_zip_extracted("cpu_instr.zip");

  assert_eq!(false, true);
}
