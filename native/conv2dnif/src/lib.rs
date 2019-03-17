#[macro_use] extern crate rustler;
//#[macro_use] extern crate rustler_codegen;
#[macro_use] extern crate lazy_static;

extern crate scoped_pool;

use rustler::{Env, Term, NifResult, Encoder};
use rustler::env::{OwnedEnv, SavedTerm};
use rustler::types::tuple::make_tuple;

mod atoms {
    rustler_atoms! {
        atom ok;
        atom error;
        //atom __true__ = "true";
        //atom __false__ = "false";
    }
}

rustler_export_nifs! {
    "Elixir.Conv2dNif",
    [("mult_n", 1, mult_n)],
    None
}

lazy_static! {
    static ref POOL:scoped_pool::Pool = scoped_pool::Pool::new(2);
}

fn mult_n<'a>(env: Env<'a>, args: &[Term<'a>]) -> NifResult<Term<'a>> {
    let pid = env.pid();
    let mut my_env = OwnedEnv::new();

    match args[0].is_list() || args[0].is_empty_list() {
        true => {
            let saved_list = my_env.run(|env| -> NifResult<SavedTerm> {
                let list_arg = args[0].in_env(env);
                Ok(my_env.save(make_tuple(env, &[list_arg])))
            })?;
            POOL.spawn(move || {
                my_env.send_and_clear(&pid, |env| {
                    let result: NifResult<Term> = (|| {
                        let list = saved_list.load(env).decode::<(Term)>()?;
                        Ok(list.decode::<Vec<Term>>()?.iter()
                                .map(|&x| match x.decode::<(f64, f64)>() {
                                    Err(_err) => std::f64::NAN,
                                    Ok(a)  => a.0 * a.1,
                                }).collect::<Vec<f64>>().encode(env))
                    })();
                    match result {
                        Err(_err) => env.error_tuple("failed".encode(env)),
                        Ok(term) => term
                    }
                });
            });
            Ok(atoms::ok().to_term(env))
        },
        false => Ok(atoms::error().to_term(env)),
    }
}
