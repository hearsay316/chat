### web 开发

关于"boring-sys*" 在windows 上的编译
#### 1 需要安装 Windows 10 sdk (10.0.26100.0)
#### 2 需要安装cmake  https://cmake.org/download/
#### 3 安装nasm  https://www.wikihow.com/Run-NASM-on-Windows
#### 4 需要安装 llvm  https://github.com/llvm/llvm-project/releases
   # Note: to emulate boringssl, "default-features = false" is required in addition to "pure-rust"

openssl*

openssl genpkey -algorithm ed25519 -out chat_server\fixtures\encoding.pem
openssl pkey -in chat_server\fixtures\encoding.pem  -pubout -out  chat_server\fixtures\decoding.pem