cargo check
cargo test 
cargo fmt --all



### 跨平台编译
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-musl --release
cp target/x86_64-unknown-linux-musl/release/claw /usr/local/bin/claw






echo  $DASHSCOPE_API_KEY
echo $ANTHROPIC_MODEL
claw "帮我看看这个项目的内容"