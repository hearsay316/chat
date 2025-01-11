### web 开发

关于"boring-sys*" 在windows 上的编译
#### 1 需要安装 Windows 10 sdk (10.0.26100.0)
#### 2 需要安装cmake  https://cmake.org/download/
#### 3 安装nasm  https://www.wikihow.com/Run-NASM-on-Windows
#### 4 需要安装 llvm  https://github.com/llvm/llvm-project/releases
   # Note: to emulate boringssl, "default-features = false" is required in addition to "pure-rust"
关于 "aws-lc-sys" 在windows 上的编译(axum 源码编译)
#### 1 C:\Program Files\OpenSSL-Win64\lib下面要有 libeay32.lib 和 ssleay32.lib (在C:\Program Files\OpenSSL-Win64\lib\VC\x64\MD下面有)
#### 2 是安装nasm 要在2.15版本之上 安装nasm  https://www.nasm.us/

openssl*

openssl genpkey -algorithm ed25519 -out chat_server\fixtures\encoding.pem
openssl pkey -in chat_server\fixtures\encoding.pem  -pubout -out  chat_server\fixtures\decoding.pem
### 已经实现
- utoipa 剩下的代码支持
- notify-server bug: 如果用户退出 SSE 连接，Dashmap 里还有这个用户的 sender (已经不能工作),
-  请帮忙 fix - 如果用户退出，则删除 Dashmap 里对应的 entry
- chat service 里未完成的 API，请帮忙完成
- 拓展 notify service，使其能够通知: (a) chat name update