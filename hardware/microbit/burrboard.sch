EESchema Schematic File Version 4
EELAYER 30 0
EELAYER END
$Descr A4 11693 8268
encoding utf-8
Sheet 1 1
Title "Drogue IoT BurrBoard"
Date "2021-12-19"
Rev "v1.0"
Comp "Red Hat"
Comment1 ""
Comment2 "https://apache.org/licenses/LICENSE-2.0"
Comment3 "License: Apache 2.0"
Comment4 "Author: Ulf Lilleengen"
$EndDescr
$Comp
L burrboard:microbit_edge_connector J2
U 1 1 61BF1E7C
P 7050 4200
F 0 "J2" H 6506 4008 50  0000 R CNN
F 1 "microbit_edge_connector" H 6506 4099 50  0000 R CNN
F 2 "burrboard:4UCON_17909_02x401.27mm_Vertical" H 6950 4100 50  0001 C CNN
F 3 "https://www.microbit.co.uk/device/pins" H 6950 4100 50  0001 C CNN
	1    7050 4200
	-1   0    0    1   
$EndComp
$Comp
L burrboard:TMP36GT9Z U1
U 1 1 61BFDC6C
P 4550 4000
F 0 "U1" H 4423 4053 60  0000 R CNN
F 1 "TMP36GT9Z" H 4423 3947 60  0000 R CNN
F 2 "burrboard:TO-92-3" H 4750 4200 60  0001 L CNN
F 3 "https://www.analog.com/media/en/technical-documentation/data-sheets/TMP35_36_37.pdf" H 4750 4300 60  0001 L CNN
F 4 "TMP36GT9Z-ND" H 4750 4400 60  0001 L CNN "Digi-Key_PN"
F 5 "TMP36GT9Z" H 4750 4500 60  0001 L CNN "MPN"
F 6 "Sensors, Transducers" H 4750 4600 60  0001 L CNN "Category"
F 7 "Temperature Sensors - Analog and Digital Output" H 4750 4700 60  0001 L CNN "Family"
F 8 "https://www.analog.com/media/en/technical-documentation/data-sheets/TMP35_36_37.pdf" H 4750 4800 60  0001 L CNN "DK_Datasheet_Link"
F 9 "/product-detail/en/analog-devices-inc/TMP36GT9Z/TMP36GT9Z-ND/820404" H 4750 4900 60  0001 L CNN "DK_Detail_Page"
F 10 "SENSOR ANALOG -40C-125C TO92-3" H 4750 5000 60  0001 L CNN "Description"
F 11 "Analog Devices Inc." H 4750 5100 60  0001 L CNN "Manufacturer"
F 12 "Active" H 4750 5200 60  0001 L CNN "Status"
	1    4550 4000
	1    0    0    -1  
$EndComp
$Comp
L Device:Q_Photo_NPN Q1
U 1 1 61C07230
P 5050 2300
F 0 "Q1" H 5240 2346 50  0000 L CNN
F 1 "Q_Photo_NPN" H 5240 2255 50  0000 L CNN
F 2 "burrboard:WP3DP3BT" H 5250 2400 50  0001 C CNN
F 3 "https://www.kingbrightusa.com/images/catalog/SPEC/WP3DP3BT.pdf" H 5050 2300 50  0001 C CNN
	1    5050 2300
	1    0    0    -1  
$EndComp
Wire Wire Line
	5650 4000 5650 3100
$Comp
L Device:R R1
U 1 1 61C212D9
P 5150 2750
F 0 "R1" H 5220 2796 50  0000 L CNN
F 1 "25K" H 5220 2705 50  0000 L CNN
F 2 "Resistor_THT:R_Axial_DIN0309_L9.0mm_D3.2mm_P2.54mm_Vertical" V 5080 2750 50  0001 C CNN
F 3 "~" H 5150 2750 50  0001 C CNN
	1    5150 2750
	1    0    0    -1  
$EndComp
Wire Wire Line
	5150 2900 5150 3100
Wire Wire Line
	6550 3400 6350 3400
Wire Wire Line
	6550 2500 6100 2500
Wire Wire Line
	6550 3500 6350 3500
Wire Wire Line
	6350 3600 6550 3600
Wire Wire Line
	6550 3700 6350 3700
Wire Wire Line
	6350 3800 6550 3800
$Comp
L Connector_Generic:Conn_01x06 J7
U 1 1 61D09808
P 6150 3700
F 0 "J7" H 6068 3175 50  0000 C CNN
F 1 "Conn_01x06" H 6068 3266 50  0000 C CNN
F 2 "Connector_PinHeader_2.54mm:PinHeader_1x06_P2.54mm_Vertical" H 6150 3700 50  0001 C CNN
F 3 "~" H 6150 3700 50  0001 C CNN
	1    6150 3700
	-1   0    0    1   
$EndComp
$Comp
L Connector_Generic:Conn_01x03 J4
U 1 1 61D0BD6C
P 6150 4400
F 0 "J4" H 6068 4075 50  0000 C CNN
F 1 "SPI" H 6068 4166 50  0000 C CNN
F 2 "Connector_PinHeader_2.54mm:PinHeader_1x03_P2.54mm_Vertical" H 6150 4400 50  0001 C CNN
F 3 "~" H 6150 4400 50  0001 C CNN
	1    6150 4400
	-1   0    0    1   
$EndComp
$Comp
L Connector_Generic:Conn_01x02 J8
U 1 1 61D56B14
P 6300 5400
F 0 "J8" H 6218 5075 50  0000 C CNN
F 1 "I2C" H 6218 5166 50  0000 C CNN
F 2 "Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical" H 6300 5400 50  0001 C CNN
F 3 "~" H 6300 5400 50  0001 C CNN
	1    6300 5400
	-1   0    0    1   
$EndComp
Wire Wire Line
	6500 5300 6550 5300
Wire Wire Line
	6500 5400 6550 5400
$Comp
L power:GND #PWR0105
U 1 1 61C5B0A7
P 4550 4650
F 0 "#PWR0105" H 4550 4400 50  0001 C CNN
F 1 "GND" H 4555 4477 50  0000 C CNN
F 2 "" H 4550 4650 50  0001 C CNN
F 3 "" H 4550 4650 50  0001 C CNN
	1    4550 4650
	1    0    0    -1  
$EndComp
$Comp
L power:GND #PWR0107
U 1 1 61C5BE7A
P 5150 3100
F 0 "#PWR0107" H 5150 2850 50  0001 C CNN
F 1 "GND" H 5155 2927 50  0000 C CNN
F 2 "" H 5150 3100 50  0001 C CNN
F 3 "" H 5150 3100 50  0001 C CNN
	1    5150 3100
	1    0    0    -1  
$EndComp
$Comp
L power:GND #PWR0115
U 1 1 61CBAE2E
P 4500 6450
F 0 "#PWR0115" H 4500 6200 50  0001 C CNN
F 1 "GND" H 4505 6277 50  0000 C CNN
F 2 "" H 4500 6450 50  0001 C CNN
F 3 "" H 4500 6450 50  0001 C CNN
	1    4500 6450
	1    0    0    -1  
$EndComp
$Comp
L Device:CP1 C1
U 1 1 61D2FCBB
P 3450 4000
F 0 "C1" H 3565 4046 50  0000 L CNN
F 1 "100nF" H 3565 3955 50  0000 L CNN
F 2 "Capacitor_THT:C_Disc_D7.0mm_W2.5mm_P5.00mm" H 3450 4000 50  0001 C CNN
F 3 "~" H 3450 4000 50  0001 C CNN
	1    3450 4000
	1    0    0    -1  
$EndComp
Wire Wire Line
	5150 2500 5150 2600
Wire Wire Line
	6100 2500 6100 2600
Connection ~ 5150 2600
Wire Wire Line
	3450 3350 3450 3850
Wire Wire Line
	3450 3350 4550 3350
Wire Wire Line
	4550 3700 4550 3350
$Comp
L power:VDD #PWR0113
U 1 1 61C961C8
P 4500 5600
F 0 "#PWR0113" H 4500 5450 50  0001 C CNN
F 1 "VDD" H 4515 5773 50  0000 C CNN
F 2 "" H 4500 5600 50  0001 C CNN
F 3 "" H 4500 5600 50  0001 C CNN
	1    4500 5600
	1    0    0    -1  
$EndComp
$Comp
L power:VDD #PWR0109
U 1 1 61C5C5FA
P 4900 4550
F 0 "#PWR0109" H 4900 4400 50  0001 C CNN
F 1 "VDD" H 4915 4723 50  0000 C CNN
F 2 "" H 4900 4550 50  0001 C CNN
F 3 "" H 4900 4550 50  0001 C CNN
	1    4900 4550
	1    0    0    -1  
$EndComp
Wire Wire Line
	3450 4150 3450 4450
Wire Wire Line
	3450 4450 4550 4450
Wire Wire Line
	4550 4300 4550 4450
Wire Wire Line
	4550 4450 4550 4650
Connection ~ 4550 4450
$Comp
L Connector_Generic:Conn_01x02 J6
U 1 1 61EBF7C6
P 6000 5100
F 0 "J6" H 5918 4775 50  0000 C CNN
F 1 "3v3" H 6050 5200 50  0000 C CNN
F 2 "Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical" H 6000 5100 50  0001 C CNN
F 3 "~" H 6000 5100 50  0001 C CNN
	1    6000 5100
	-1   0    0    1   
$EndComp
$Comp
L Connector_Generic:Conn_01x02 J9
U 1 1 61F033C4
P 5950 5700
F 0 "J9" H 5868 5375 50  0000 C CNN
F 1 "GND" H 5868 5466 50  0000 C CNN
F 2 "Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical" H 5950 5700 50  0001 C CNN
F 3 "~" H 5950 5700 50  0001 C CNN
	1    5950 5700
	-1   0    0    1   
$EndComp
Text Label 5150 1700 0    50   ~ 0
POWER
Text Label 4550 3150 0    50   ~ 0
POWER
$Comp
L power:GND #PWR01
U 1 1 61F8383A
P 6050 6900
F 0 "#PWR01" H 6050 6650 50  0001 C CNN
F 1 "GND" H 6055 6727 50  0000 C CNN
F 2 "" H 6050 6900 50  0001 C CNN
F 3 "" H 6050 6900 50  0001 C CNN
	1    6050 6900
	1    0    0    -1  
$EndComp
Wire Wire Line
	6050 5800 6050 6300
NoConn ~ 6550 5900
NoConn ~ 6550 6000
NoConn ~ 6550 5500
Wire Wire Line
	6350 3900 6550 3900
NoConn ~ 6550 4000
NoConn ~ 6550 4100
NoConn ~ 6550 4200
NoConn ~ 6550 4600
NoConn ~ 6550 4700
Wire Wire Line
	6200 5100 6550 5100
Wire Wire Line
	6550 5000 6200 5000
NoConn ~ 6550 5200
Wire Wire Line
	5650 3100 6550 3100
NoConn ~ 6550 3000
NoConn ~ 6550 3200
NoConn ~ 6550 3300
NoConn ~ 6550 2900
NoConn ~ 6550 2800
NoConn ~ 6550 2700
NoConn ~ 6550 2600
NoConn ~ 6550 2400
NoConn ~ 6550 2300
NoConn ~ 6550 2200
NoConn ~ 6550 2100
Wire Wire Line
	6350 4300 6550 4300
Wire Wire Line
	6550 4400 6350 4400
Wire Wire Line
	6350 4500 6550 4500
Wire Wire Line
	5150 1700 5150 2100
Wire Wire Line
	4550 3150 4550 3350
Connection ~ 4550 3350
Text Label 5700 4450 0    50   ~ 0
POWER
$Comp
L power:PWR_FLAG #FLG01
U 1 1 6206B779
P 5000 5000
F 0 "#FLG01" H 5000 5075 50  0001 C CNN
F 1 "PWR_FLAG" H 5000 5173 50  0000 C CNN
F 2 "" H 5000 5000 50  0001 C CNN
F 3 "~" H 5000 5000 50  0001 C CNN
	1    5000 5000
	1    0    0    -1  
$EndComp
$Comp
L power:PWR_FLAG #FLG02
U 1 1 6206BF07
P 6050 6300
F 0 "#FLG02" H 6050 6375 50  0001 C CNN
F 1 "PWR_FLAG" V 6050 6428 50  0000 L CNN
F 2 "" H 6050 6300 50  0001 C CNN
F 3 "~" H 6050 6300 50  0001 C CNN
	1    6050 6300
	0    1    1    0   
$EndComp
Connection ~ 6050 6300
Wire Wire Line
	6050 6300 6050 6900
Wire Wire Line
	6150 5600 6400 5600
Wire Wire Line
	6150 5700 6300 5700
Wire Wire Line
	6050 5800 6300 5800
Wire Wire Line
	6300 5800 6300 5700
Connection ~ 6300 5700
Wire Wire Line
	6300 5700 6400 5700
Wire Wire Line
	6400 5700 6400 5600
Connection ~ 6400 5700
Wire Wire Line
	6400 5700 6550 5700
Connection ~ 6400 5600
Wire Wire Line
	6400 5600 6550 5600
NoConn ~ 6550 5800
$Comp
L power:PWR_FLAG #FLG0101
U 1 1 62076CDD
P 5700 4550
F 0 "#FLG0101" H 5700 4625 50  0001 C CNN
F 1 "PWR_FLAG" H 5700 4723 50  0000 C CNN
F 2 "" H 5700 4550 50  0001 C CNN
F 3 "~" H 5700 4550 50  0001 C CNN
	1    5700 4550
	0    -1   -1   0   
$EndComp
Wire Wire Line
	5150 2600 6100 2600
$Comp
L Switch:SW_SPDT SW1
U 1 1 61D46698
P 5300 5000
F 0 "SW1" H 5300 5285 50  0000 C CNN
F 1 "POWER_SW" H 5400 5200 50  0000 C CNN
F 2 "burrboard:Switch_Slide_11.6x4mm_EG1218" H 5300 5000 50  0001 C CNN
F 3 "~" H 5300 5000 50  0001 C CNN
	1    5300 5000
	1    0    0    -1  
$EndComp
Wire Wire Line
	5500 4900 6550 4900
Wire Wire Line
	5100 5000 5000 5000
Wire Wire Line
	4900 5000 4900 4550
Connection ~ 5000 5000
Wire Wire Line
	5000 5000 4900 5000
NoConn ~ 5500 5100
Wire Wire Line
	4950 4000 5650 4000
$Comp
L Connector_Generic:Conn_01x02 J1
U 1 1 61D64CAB
P 5900 4600
F 0 "J1" H 5818 4275 50  0000 C CNN
F 1 "SENSORS" V 6000 4550 50  0000 C CNN
F 2 "Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical" H 5900 4600 50  0001 C CNN
F 3 "~" H 5900 4600 50  0001 C CNN
	1    5900 4600
	0    -1   -1   0   
$EndComp
Wire Wire Line
	5900 4800 5700 4800
Wire Wire Line
	5700 4800 5700 4550
Connection ~ 5700 4550
Wire Wire Line
	5700 4550 5700 4450
Wire Wire Line
	6000 4800 6550 4800
$Comp
L Device:Battery BT1
U 1 1 61D45279
P 4500 6000
F 0 "BT1" H 4608 6046 50  0000 L CNN
F 1 "Battery" H 4608 5955 50  0000 L CNN
F 2 "Battery:BatteryHolder_Keystone_2468_2xAAA" V 4500 6060 50  0001 C CNN
F 3 "https://www.keyelco.com/userAssets/file/M65p28.pdf" V 4500 6060 50  0001 C CNN
	1    4500 6000
	1    0    0    -1  
$EndComp
Wire Wire Line
	4500 5600 4500 5800
Wire Wire Line
	4500 6200 4500 6450
$EndSCHEMATC
