use oxeylyzer_repl::repl::Repl;

use std::io;

fn main() -> io::Result<()>
{
    let app_result = Repl::new().run();

    return app_result;
}
