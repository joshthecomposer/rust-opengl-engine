
#[macro_export]
macro_rules! gl_call {
    ($e:expr) => {{
        while gl::GetError() != gl::NO_ERROR {} // Clear all existing OpenGL errors
        let result = $e; // Execute the OpenGL call
        $crate::macros::gl_log_call(stringify!($e), file!(), line!()); // Log any errors
        result
    }};
}

pub fn gl_log_call(function: &str, file: &str, line: u32) -> bool {
    unsafe {
        let mut had_error = false;
        loop {
            let err = gl::GetError();
            if err == gl::NO_ERROR {
                break; // Exit if no more errors
            }
            had_error = true;
            eprintln!(
                "[OpenGL Error] ({}) in function `{}`, at {}:{}",
                err, function, file, line
            );
        }
        had_error
    }
}
