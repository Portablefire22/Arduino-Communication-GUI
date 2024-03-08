#include <ArduinoBLE.h>

#define array_length(x) (sizeof(x) / sizeof(x[0]))

const char uuid[] = "4315b8fb-7cca-4ba6-a4c0-c3c0c915180f"; //specific to the intended target nano system
const char local[] = "4315b8fb-7cca-4ba6-a4c0-c3c0c915180f"; // giving the characteristic a specific local name "local"
const char newtonChar[] = "c3ccbb8e-930c-4add-b57a-ce692b0c36ae"; // giving the characteristic a specific local name "local"

uint16_t temp_count = 0;
bool temp_bool = true;
struct Packet {
    uint8_t PacketKind; // What type of packet it is
    uint8_t PacketId;
    int8_t RawData[32];     // Raw bytes of data, 2 bytes taken for first two args, 
                            // one byte for end of text, leaving 29 bytes for data per communication
    
  };

class PacketHandler {
  private:
    
  public: 
  
  /* ID types: 
    1: String 
    2: PosInteger
    3: NegInteger
    4: Binary
  */
  void send_packet(Packet packet) {
    uint8_t data_to_send[32];
    data_to_send[0] = packet.PacketKind;
    data_to_send[1] = packet.PacketId;
    for (int i=2; i < 32; i++) {
      data_to_send[i] = packet.RawData[i - 2];
      if (packet.RawData[i-2] == 0x0D) {
        break;
      }
    }
    data_to_send[31] = 0x0D; // Fail safe

    Serial.write(data_to_send, 32); // 32 seems to be the magic number for outputting it all in one go
    
  }

  // Wipes data in buffer and sets it to given, also ends early if possible
  // Takes pointed to first element and the size of the array
  void set_data(Packet* packet, uint8_t* data, uint8_t data_size) {
    for (int i = 0; i < data_size; i++) {
      packet->RawData[i] = data[i];
    }
    packet->RawData[data_size] = 0x0D;
  }

  // Inserts the int16_t as a byte into the provided array
  void convert_u16(int16_t in, uint8_t data[2]) {
    data[0] = in & 0xff;
    data[1] = (in >> 8);
  }

  Packet create_packet(uint8_t packet_kind, uint8_t packet_id){
    struct Packet packet;
    packet.PacketKind = packet_kind;
    packet.PacketId = packet_id;
    packet.RawData[32] = {0}; 
    return packet;
  }

  void serialFlush() {
    while(Serial.available() > 0) {
      char t = Serial.read();
    }
  }
};

PacketHandler* packet_handler = new PacketHandler();
Packet pack;
void setup() {
  Serial.begin(9600);
  pinMode(13, OUTPUT);

  //
  // begin initialization
  //
  if (!BLE.begin()) {
    Serial.println("starting Bluetooth® Low Energy module failed!");
    while (1);
  }
  Serial.println("BLE scan for a peripheral [sender] device...");
  //
  // start scanning for peripheral
  //
  BLE.scanForUuid(uuid);  // this matches the sender - must be unique

}

void loop() {
  // check if a peripheral has been discovered
  BLEDevice peripheral = BLE.available();

  if (peripheral) {
    //this section of code prints out the main details of the sender
    Serial.println("Discovered a peripheral - details below");
    
    Serial.print("Address: ");
    Serial.println(peripheral.address()); //using a BLE function

    // print the local name, if present
    if (peripheral.hasLocalName()) {
      Serial.print("Sender Name: ");
      Serial.println(peripheral.localName());
    }

    // print the advertised service UUIDs, if present
    if (peripheral.hasAdvertisedServiceUuid()) {
      Serial.print("Service UUID: ");
      for (int i = 0; i < peripheral.advertisedServiceUuidCount(); i++) {
        Serial.print(peripheral.advertisedServiceUuid(i));
        Serial.print(" ");
      }
      Serial.println();
    }

    // print the RSSI
    Serial.print("RSSI: ");
    Serial.println(peripheral.rssi());

    Serial.println();
    BLE.stopScan(); //once you have "acquired" the sender, stop scanning

    Serial.println("Connecting ...");

    if (peripheral.connect()) {
      Serial.println("Connected");

      Serial.println("- Discovering peripheral device attributes, wait 20s...");//need to discover the attributes of the sender
      if (peripheral.discoverAttributes()) {
        Serial.println("* Peripheral device attributes discovered!");
        Serial.println(" ");
      } else {
        Serial.println("* Peripheral device attributes discovery failed!");
        Serial.println(" ");
        peripheral.disconnect();
        return;
      }

      if (peripheral) {
        // this section simply counts the number of characteristcis it sees. 
        //It was used to see if there was actually a charcteristic being broadcast
        int characteristicCount = peripheral.characteristicCount(); //using a BLE function

        //Serial.print(characteristicCount);
        Serial.println("characteristics discovered in service");
      } else {
        Serial.println("Peripheral does NOT have service");
      }

      //now we make a local name for the characteristic on this receiver called localp
      //local was setup earlier with the correct uuid
      BLECharacteristic localp = peripheral.characteristic(local);
      BLECharacteristic localNewton = peripheral.characteristic(newtonChar);
      
      //following lines were inserted to show if the characteristic was functional
      if (!localp) {
        Serial.println("* Peripheral device does not have characteristic!");
        peripheral.disconnect();
        return;
      } else if (!localp.canWrite()) {
        Serial.println("Peripheral does not have a writable LED characteristic!");
        peripheral.disconnect();
        return;
      } else{
        //shows that localp now has the correct uuid
        Serial.println(localp.uuid());
      }
      
      int16_t revolutions = 0;
      float newtons = 0.0;
      uint8_t data[2];
      char str[28];
      while (peripheral.connected()) {
        Serial.println("Reading the characteristic");
        localp.readValue(&revolutions, sizeof(revolutions)); //needs to know the value and the size of the data
        localNewton.readValue(&newtons, sizeof(newtons));
        packet_handler->convert_u16(revolutions, data);
        if (revolutions < 0) {
          pack = packet_handler->create_packet(3, 0);
        } else {
          pack = packet_handler->create_packet(2, 0);
        }
        uint8_t size = sizeof(data) / sizeof(data[0]);
        packet_handler->set_data(&pack, data, size);
        packet_handler->send_packet(pack);

        snprintf(str, 28, "%f", newtons);
        pack = packet_handler->create_packet(4, 1);
        packet_handler->set_data(&pack, str, array_length(str));
      } 

    } else {
      Serial.println("Failed to connect!");
      return;
    }
  }
}
