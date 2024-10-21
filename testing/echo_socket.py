import socket


def main() -> None:
    while True:
        sock: socket.socket = socket.socket(socket.AF_INET6, socket.SOCK_STREAM)
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        sock.bind(("::1", 42069))
        sock.listen()
        while True:
            try:
                conn, _ = sock.accept()
                recv: bytes = conn.recv(4096)
                conn.sendall(recv)
                conn.close()
                return
            except KeyboardInterrupt:
                sock.close()
                return
            except:
                pass


if __name__ == "__main__":
    main()
