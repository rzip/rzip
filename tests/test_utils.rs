use std::fs::{create_dir_all, remove_dir_all};
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

pub mod tests {
    use async_std::task;
    use std::future::Future;

    pub struct TestCleaner<S = Box<dyn Fn(&str) -> ()>, T = Box<dyn Fn(&str) -> ()>>
    where
        S: Fn(&str) -> (),
        T: Fn(&str) -> (),
    {
        setup: S,
        teardown: T,
    }

    impl TestCleaner {
        pub fn new<S: 'static, T: 'static>(setup: S, teardown: T) -> Self
        where
            S: Fn(&str) -> (),
            T: Fn(&str) -> (),
        {
            TestCleaner {
                setup: Box::new(setup),
                teardown: Box::new(teardown),
            }
        }

        pub fn run<R>(&self, test: R)
        where
            R: FnOnce(String) -> Box<dyn Future<Output = ()>> + Send + std::panic::UnwindSafe,
        {
            let folder_name = super::random_name();

            (self.setup)(folder_name.as_ref());

            let result = std::panic::catch_unwind(|| {
                task::block_on(async {
                    std::pin::Pin::from(test(folder_name.clone())).await;
                });
            });

            (self.teardown)(folder_name.as_ref());

            result.unwrap();
        }
    }
}

fn random_name() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;

    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(25)
        .collect::<String>()
}

fn delete_temp_dir(folder_name: &str) {
    let path = PathBuf::from(format!("tests/temp/{}", folder_name));
    let result = remove_dir_all(&path);

    println!("Cleaning up tests... {:?} - {:?}", path, result);

    match result {
        Err(ref e) if e.kind() == NotFound => {}
        Err(ref e) if e.kind() != NotFound => result.unwrap(),
        _ => {}
    }
}

pub fn build_basic_runner() -> tests::TestCleaner {
    tests::TestCleaner::new(
        |folder_name| {
            delete_temp_dir(folder_name);
            create_dir_all(format!("tests/temp/{}", folder_name)).unwrap();
        },
        |folder_name| {
            delete_temp_dir(folder_name);
        },
    )
}

/*#[test]
fn test_test_unitilities() {
    let runner = build_basic_runner();

    runner.run(|_folder_name| {
        panic!();
    })
}*/
