#define array_length(x) (sizeof(x) / sizeof(x[0]))

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
  void convert_u16(int16_t in, int8_t data[2]) {
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
}

void loop() {
  //packet_handler->send_packet(pack);
  int16_t t = -5325;
  int8_t data[2];
  packet_handler->convert_u16(t, data);
  pack = packet_handler->create_packet(1, 0);
  packet_handler->set_data(&pack, "Testing", array_length("Testing"));
  packet_handler->send_packet(pack);
  delay(10);
  if (temp_bool) {
    packet_handler->convert_u16(500, data);
    pack = packet_handler->create_packet(2, 1);
    packet_handler->set_data(&pack, data, array_length(data));
  } else {
    packet_handler->convert_u16(-500, data);
    pack = packet_handler->create_packet(3, 1);
    packet_handler->set_data(&pack, data, array_length(data));
  }
  temp_bool = !temp_bool;
  packet_handler->send_packet(pack);
  delay(1000);
}

// arduino-cli compile --fqbn arduino:avr:uno testSketch && arduino-cli upload testSketch -p /dev/ttyUSB0 -b arduino:avr:uno    
