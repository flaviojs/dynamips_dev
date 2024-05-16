use dynamips_c::_private::*;
use dynamips_c::udp_recv::*;

fn main() {
    let args: Vec<_> = std::env::args().map(|x| CString::new(x).unwrap()).collect();
    if args.len() < 3 {
        println!("Usage: udp_recv <output_file> <udp_port>");
        std::process::exit(libc::EXIT_FAILURE);
    }
    let args: Vec<*mut c_char> = args.iter().map(|x| x.as_ptr().cast_mut()).collect();
    std::process::exit(unsafe { udp_recv_main(args.len() as c_int, args.as_ptr().cast_mut()) });
}
