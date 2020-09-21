//! Load and run the no-std binary
use std::{fs::File, path::Path, process::Command};

fn get_kernel_path() -> impl AsRef<Path> {
    // Ideally this function would just return env_path. However, this path
    // currently points to the kernel built for the host target, rather than
    // our custom target. So we have to modify the path the insert the
    // necessary path component.
    let env_path = Path::new(env!("CARGO_BIN_EXE_kernel"));

    let mut env_comps = env_path.components();
    let bin_name = env_comps.next_back().unwrap();
    let bin_dir = env_comps.next_back().unwrap();
    let mut pathbuf = env_comps.as_path().to_path_buf();

    pathbuf.push("x86_64-none-uk");
    pathbuf.push(bin_dir);
    pathbuf.push(bin_name);
    pathbuf
}

#[test]
fn qemu_integration_test() {
    let kernel_path = get_kernel_path();
    let kernel = File::open(&kernel_path).unwrap();
    let length = kernel.metadata().unwrap().len();
    drop(kernel);

    println!("Kernel has length: {}", length);

    let mut c = Command::new("qemu-system-x86_64");
    c.args(&[
        "-machine",
        "type=q35,accel=kvm",
        "-cpu",
        "host,-vmx",
        "-smp",
        "cpus=1",
        "-nodefaults",
        "-display",
        "none",
        "-serial",
        "stdio",
        "-kernel",
        kernel_path.as_ref().to_str().unwrap(),
    ]);

    println!("Spawning: {:?}", c);
    let res = c.status().expect("Failed to launch QEMU");
    println!("QEMU exited with: {}", res);
}
