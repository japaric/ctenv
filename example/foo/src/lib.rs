use ctenv_macros::ctenv;

pub static BUFFER: [u8; ctenv!(BUF_SZ)] = [0; ctenv!(BUF_SZ)];
