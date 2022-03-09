
```
  $ sudo dnf upgrade --refresh -y
  $ sudo dnf install bluez-mesh
  $ sudo dnf install curl dnf-plugins-core openssl-devel npm gcc gcc-c++ clang make cyrus-sasl-devel cmake podman podman-docker
  $ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  $ cargo install drg
  $ cargo install --features=ssl websocat
  $ git clone https://github.com/bluez/bluez.git
  $ pip install numpy dbus-python termcolor
  $ sudo service bluetooth stop
  $ sudo service bluetooth-mesh start
  $ ./bluez/test/test-mesh

```

If `mesh-cfgclient` silently fails, `mkdir ~/.config` and retry.

To view the mesh daemon logs: `journalctl -u bluetooth-mesh -f`

To add `--debug` to the `ExecStart` command:

```
  $ systemctl edit bluetooth-mesh.service
  $ systemctl restart bluetooth-mesh
```
