# Relayer

- 配置文件

    配置文件位于 `$HOME/.config/relayer`, 客户端和服务器配置文件分别为 `relayc.json` 和 `relays.json`.

    配置样例参见 examples 目录, lhost 和 lport 表示本地监听地址,
    rhost 和 rport 表示远程要连接的地址.

- 开发测试

    - 本地服务器

        在远程服务器上执行

            cargo run --bin relays

        输出

            [+] Listening at 127.0.0.1:8001 ...

    - 客户端

        在本地运行

            cargo run --bin relayc

        输出

            [+] Listening at 127.0.0.1:8000 ...

    - 本地测试命令

            curl --socks5 127.0.0.1:8000 https://www.baidu.com
