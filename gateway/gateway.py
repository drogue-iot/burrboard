#!/usr/bin/env python3
import blemesh
import json
try:
  from gi.repository import GLib
except ImportError:
  import glib as GLib
from dbus.mainloop.glib import DBusGMainLoop


import sys
import struct
import fcntl
import os
import numpy
import random
import dbus
import dbus.service
import dbus.exceptions

import paho.mqtt.client as mqtt
import time
import ssl
import os
import json
import logging


########################
# On Off Server Model
########################
class GatewayOnOffServer(blemesh.ServerModel):
	def __init__(self, name, model_id):
		blemesh.Model.__init__(self, model_id)
		self.name = name
		self.tid = None
		self.last_src = 0x0000
		self.last_dst = 0x0000
		self.cmd_ops = { 0x8201,  # get
				 0x8202,  # set
				 0x8203,  # set unacknowledged
				 0x8204 } # status

		self.state = 0
		blemesh.log.info("OnOff Server: " + blemesh.get_state_str(self.state))
		self.pub_timer = blemesh.ModTimer()
		self.t_timer = blemesh.ModTimer()

	def process_message(self, source, dest, key, data):
		datalen = len(data)
		if datalen != 3:
			# The opcode is not recognized by this model
			return

		opcode, state = struct.unpack('>HB',bytes(data))
		if opcode != 0x8204 :
			# The opcode is not recognized by this model
			return

		state_str = blemesh.get_state_str(state)
		device = '%04x' % source
		topic = self.name + "/" + device

		blemesh.log.info("Sending state '" + state_str + "' from device '" + device + "' to MQTT topic '" + topic + "'")
		#TODO Handle failures
		client.publish(topic, "{state:" + state_str + "}")

	def publish(self):
		data = struct.pack('>HB', 0x8204, self.state)
		self.send_publication(data)

########################
# Sensor Server Model
########################
class GatewaySensorServer(blemesh.BurrBoardSensorServer):
	def __init__(self, name, model_id):
		blemesh.Model.__init__(self, model_id)
		self.name = name
	def process_message(self, source, dest, key, data):
		blemesh.log.info('gateway sensor model process message')
		sensor_data = self.parse_sensor_data(data)
		blemesh.log.info('sensor data parsed')
		if sensor_data != None and len(sensor_data) != 0:
			device = '%04x' % source
			topic = self.name + "/" + device
			blemesh.log.info("Sending state '" + json.dumps(sensor_data) + "' from device '" + device + "' to MQTT topic '" + topic + "'")
			#TODO Handle failures
			client.publish(topic, json.dumps(sensor_data))


def on_connect(client, userdata, flags, rc):
	if rc == 0:
		blemesh.log.info("Connected to Drogue cloud!")
		client.subscribe("command/inbox/#")
		client.on_message = on_message
	else:
		blemesh.log.error("Failed to connect, return code %d\n", rc)

def on_publish(client, userdata, result):
	blemesh.log.info("Published to cloud")

def on_message(client, userdata, msg):
	blemesh.log.info("Received command:" + msg.topic + " - " + str(msg.payload))

	segments = msg.topic.split("/")
	if len(segments) != 4:
		blemesh.log.error("Topic %s Not properly formatted", msg.topic)
		return

	if segments[3] != "set-state":
		blemesh.log.error("Command %s not recognized", segments[3])
		return

	device = int(segments[2], 16)

	state = -1
	command = json.loads(msg.payload.decode("utf-8"))
	if not command["state"] is None:
		if command["state"].lower() == "on":
			state = 1
		elif command["state"].lower() == "off":
			state = 0
		else:
			state = -1

	blemesh.log.info("Setting device '" + str('%04x' % device) + "' state to '" + command["state"] + "'")

	if state != -1:
		blemesh.app.elements[1].models[0].set_state(device, 0, state)

def main():
	global client
	global log
	blemesh.configure_logging("gateway")

	token = os.environ.get('TOKEN')
	if token is None:
		blemesh.log.error("'TOKEN' variable not set")
		sys.exit(1)

	blemesh.set_token(token)

	broker = os.environ.get('DROGUE_MQTT_HOST', 'mqtt.sandbox.drogue.cloud')
	port = os.environ.get('DROGUE_MQTT_PORT', 8883)

	application = os.environ.get('DROGUE_APPLICATION', 'ble-demo')
	device = os.environ.get('DROGUE_DEVICE', 'gateway')
	password = os.environ.get('DROGUE_PASSWORD', 'hey-rodney')

	blemesh.log.info('Drogue endpoint: ' + broker + ':' + str(port))
	blemesh.log.info('Drogue application: ' + application)
	blemesh.log.info('Drogue device: ' + device)

	username = device + "@" + application

	client = mqtt.Client("drogue_gateway")
	client.on_connect = on_connect
	client.on_publish = on_publish
	client.username_pw_set(username, password)
	client.tls_set(cert_reqs=ssl.CERT_NONE)
	client.connect(broker, port)
	client.loop_start()


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
	second_ele = blemesh.Element(blemesh.bus, 0x01)

	blemesh.log.info('Register OnOff Server model on element 0')
	first_ele.add_model(GatewayOnOffServer(application, 0x1000))
	first_ele.add_model(GatewaySensorServer(application, 0x1100))

	blemesh.log.info('Register Vendor model on element 0')
	first_ele.add_model(blemesh.SampleVendor(0x0001))

	blemesh.log.info('Register OnOff Client model on element 1')
	second_ele.add_model(blemesh.OnOffClient(0x1001))
	second_ele.add_model(blemesh.SensorClient(0x1102))

	blemesh.app.add_element(first_ele)
	blemesh.app.add_element(second_ele)
	blemesh.mainloop = GLib.MainLoop()

	blemesh.attach(blemesh.token)
	blemesh.mainloop.run()


if __name__ == '__main__':
	main()