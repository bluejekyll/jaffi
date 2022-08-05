java_src_path := "integration_tests/src"
java_files := replace(`cd integration_tests/src && fd --glob *.java`, "\n", " ")
target_dir := "target"
dylib_path := absolute_path(target_dir + "/debug")
dylib_name := "jaffi_integration_tests"
class_path := absolute_path(target_dir + "/classes")

build:
    cargo build --all
    cd {{java_src_path}} && javac -d {{class_path}} {{java_files}}

test: build
    JAFFI_LIB={{dylib_name}} java -Xcheck:jni -Djava.library.path={{dylib_path}} --class-path {{class_path}} net.bluejekyll.TestRunner

clean:
    rm -rf {{class_path}}
    cargo clean -p jaffi_integration_tests
 
lint:
    cargo clippy --all -- -D warnings

publish:
    cargo publish -p jaffi_support
    `sleep 10`
    cargo publish -p jaffi
