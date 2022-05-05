
```
  $ sudo dnf upgrade --refresh -y
  $ sudo dnf install bluez-mesh
  $ sudo dnf install curl dnf-plugins-core openssl-devel npm gcc gcc-c++ clang make cyrus-sasl-devel cmake podman podman-docker systemd-devel dbus-devel
  $ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  $ cargo install drg
  $ cargo install --features=ssl websocat
  $ cargo install probe-rs-cli
  $ sudo dnf install git pip
  $ git clone https://github.com/bluez/bluez.git
  $ pip install numpy dbus-python termcolor paho-mqtt
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

## Microshift

```
sudo dnf module enable -y cri-o:1.21
sudo dnf install -y cri-o cri-tools
sudo systemctl enable crio --now

sudo dnf copr enable -y @redhat-et/microshift
sudo dnf install -y microshift
sudo firewall-cmd --zone=trusted --add-source=10.42.0.0/16 --permanent
sudo firewall-cmd --zone=public --add-port=80/tcp --permanent
sudo firewall-cmd --zone=public --add-port=443/tcp --permanent
sudo firewall-cmd --zone=public --add-port=5353/udp --permanent
sudo firewall-cmd --reload
sudo systemctl enable microshift --now

curl -O https://mirror.openshift.com/pub/openshift-v4/$(uname -m)/clients/ocp/stable/openshift-client-linux.tar.gz
sudo tar -xf openshift-client-linux.tar.gz -C /usr/local/bin oc kubectl

mkdir ~/.kube
sudo cat /var/lib/microshift/resources/kubeadmin/kubeconfig > ~/.kube/config

oc get pods -A
```
