**start clean mesh (first terminal):**
<pre><code>
$ <b>rm -rf ~/.config/meshcfg/config_db.json</b>
$ <b>sudo rm -rf /var/lib/bluetooth/mesh</b>
$ <b>sudo mkdir /var/lib/bluetooth/mesh</b>
$ <b>sudo /usr/libexec/bluetooth/bluetooth-meshd -nd --debug</b>
</code></pre>


**provisioner (second terminal):**
<pre><code>
$ <b>mesh-cfgclient</b>

Warning: config file "/home/pi/.config/meshcfg/config_db.json" not found
[mesh-cfgclient]# <b>create</b>
Created new node with token 522b02d50de1c8a2
Proxy added: org.bluez.mesh.Node1 (/org/bluez/mesh/nodea186fe8c7e3b461daa29664eba26ed2d)
Proxy added: org.bluez.mesh.Management1 (/org/bluez/mesh/nodea186fe8c7e3b461daa29664eba26ed2d)
Attached with path /org/bluez/mesh/nodea186fe8c7e3b461daa29664eba26ed2d
[mesh-cfgclient]# <b>appkey-create 0 0</b>
</code></pre>

**device (third terminal):**
<pre><code>
$ <b>./device.py join</b>

<b>join</b>
Joining with UUID 3df8492249f1476491a1e4e1d051a5f7
Join procedure started
</code></pre>

**provisioner:**
<pre><code>
[mesh-cfgclient]# <b>provision 3df8492249f1476491a1e4e1d051a5f7</b>
</code></pre>


**device:**
```
PromptStatic ( static-oob )
Enter 16 octet key on remote device:  6f4b1f31cbe0d0f81b0ef64a1c70b86c
```

**provisioner:**
<pre><code>
[[mesh-agent]# ] Enter key (hex number): <b>6f4b1f31cbe0d0f81b0ef64a1c70b86c</b>
</code></pre>

**device:**
```
Joined mesh network with token 62cb5d464413e5c7
```

**provisioner:**
<pre><code>
Assign addresses for 2 elements
Provisioning done:
Mesh node:
	UUID = 3DF8492249F1476491A1E4E1D051A5F7
	primary = 00aa

	elements (2):

[mesh-cfgclient]# <b>menu config</b>
[mesh-cfgclient]# <b>target 00aa</b>
Configuring node 00aa

[config: Target = 00aa]# <b>appkey-add 0</b>

[config: Target = 00aa]# <b>composition-get</b>
Received DeviceCompositionStatus (len 29)
Received composion:
	Feature support:
		relay: yes
		proxy: no
		friend: yes
		lpn: no
	 Element 0:
		location: 0000
		SIG defined models:
		  Model ID	0000 "Configuration Server"
		  Model ID	1000 "Generic OnOff Server"
		 Vendor defined models:
		  Model ID	05f1 0001
	 Element 1:
		location: 0000
		SIG defined models:
		  Model ID	1001 "Generic OnOff Client"

[config: Target = 00aa]# <b>bind 00aa 0 1000</b>
[config: Target = 00aa]# <b>bind 00ab 0 1001</b>
[config: Target = 00aa]# <b>bind 00aa 0 1100</b>
[config: Target = 00aa]# <b>bind 00ab 0 1102</b>
</code></pre>
