//#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use tress::run;

fn main() {
    pollster::block_on(run());
}
