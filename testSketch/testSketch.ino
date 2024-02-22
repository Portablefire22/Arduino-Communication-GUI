

struct Packet {
    uint8_t PacketKind; // What type of packet it is
    uint8_t PacketId;
    uint8_t *RawData[32];     // Raw bytes of data, 2 bytes taken for first two args, one byte for end of text, leaving 29 bytes for data per communication
  };

class PacketHandler {
  private:
    
  public: 
  
  void send_packet(Packet packet) {
    uint8_t data_to_send[32];
    data_to_send[0] = packet.PacketKind;
    data_to_send[1] = packet.PacketId;
    for (int i=2; i < 32; i++) {
      data_to_send[i] = packet.RawData[i - 2];
    }
    data_to_send[31] = 0x0D;

    Serial.write(data_to_send, 32); // 32 seems to be the magic number for outputting it all in one go

  }

  void insert_data(Packet* packet) {

  }

  Packet create_packet(uint8_t packet_kind, uint8_t packet_id, uint8_t raw_data[60]){
    struct Packet packet;
    packet.PacketKind = packet_kind;
    packet.PacketId = packet_id;
    for (int i=0; i < 32; i++){
      packet.RawData[i] = raw_data[i];
    }
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
  uint8_t data[30];
  for (int i=0; i<29; i++) {
    data[i] = i;
  }
  pack = packet_handler->create_packet(4, 1, data);
}

void loop() {
  packet_handler->send_packet(pack);
  delay(1000);
}
