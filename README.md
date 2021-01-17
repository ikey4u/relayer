# Relayer

- 配置文件

    配置文件位于 `$HOME/.config/relayer/config.json`, 一个简单的配置样例参见 examples 目录.
    local 表示本地监听地址, remote 表示要连接的远程地址.

    客户端必须配置 local 和 remote, 忽略 relayer 选项.

    服务器端如果不作为中继, 则只需配置 local 即可, 如果服务器要作为中继使用,
    则必须在 remote 中配置中继服务器的地址, 且将 relayer 设置为 true.

- 启动

    在远程服务器上执行

        cargo run --bin relays

    在本地运行

        cargo run --bin relayc

    本地测试命令

        curl --socks5 127.0.0.1:8081 https://www.google.com
