#!/usr/bin/env python3
import blemesh
try:
  from gi.repository import GLib
except ImportError:
  import glib as GLib
from dbus.mainloop.glib import DBusGMainLoop
import dbus
import dbus.service
import dbus.exceptions
import sys
import os

def main():
	blemesh.configure_logging("device")

	DBusGMainLoop(set_as_default=True)
	blemesh.bus = dbus.SystemBus()

	blemesh.mesh_net = dbus.Interface(blemesh.bus.get_object(blemesh.MESH_SERVICE_NAME,
						"/org/bluez/mesh"),
						blemesh.MESH_NETWORK_IFACE)

	blemesh.mesh_net.connect_to_signal('InterfacesRemoved', blemesh.interfaces_removed_cb)

	blemesh.app = blemesh.Application(blemesh.bus)

	# Provisioning agent
	blemesh.app.set_agent(blemesh.Agent(blemesh.bus))

	first_ele = blemesh.Element(blemesh.bus, 0x00)

	blemesh.log.info('Register Sensor client model on element 0')
	first_ele.add_model(blemesh.SensorClient(0x1102))
	blemesh.log.info('Register Vendor model on element 0')
	first_ele.add_model(blemesh.FirmwareUpdateClient())
	blemesh.log.info('Register OnOff Client model on element 0')
	first_ele.add_model(blemesh.OnOffClient(0x1001))

	blemesh.app.add_element(first_ele)

	blemesh.mainloop = GLib.MainLoop()

	if (len(sys.argv) == 2):
		if sys.argv[1] == 'join':
			blemesh.join()
		else:
			blemesh.log.error("Unknown command: " + sys.argv[1])
			sys.exit(1)
	else:
		token = os.environ.get('TOKEN')
		if token is None:
			blemesh.log.error("'TOKEN' variable not set")
			sys.exit(1)

		blemesh.set_token(token)
		blemesh.attach(blemesh.token)

	blemesh.mainloop.run()


if __name__ == '__main__':
	main()
