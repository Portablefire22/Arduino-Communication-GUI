#define array_length(x) (sizeof(x) / sizeof(x[0]))

bool temp_bool = true;
int temp_count = 0;

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
  void send_packet(Packet* packet) {
    uint8_t data_to_send[32] = {0};
    data_to_send[0] = packet->PacketKind;
    data_to_send[1] = packet->PacketId;
    for (int i=2; i < 32; i++) {
      data_to_send[i] = packet->RawData[i - 2];
      if (packet->RawData[i-2] == 0x17) {
        break;
      }
    }
    data_to_send[31] = 0x17; // Fail safe

    Serial.write(data_to_send, 32); // 32 seems to be the magic number for outputting it all in one go
    
  }

  // Wipes data in buffer and sets it to given, also ends early if possible
  // Takes pointed to first element and the size of the array
  void set_data(Packet* packet, uint8_t* data, uint8_t data_size) {
    for (int i = 0; i < data_size; i++) {
      packet->RawData[i] = data[i];
    }
    packet->RawData[data_size] = 0x17;
  }

  void set_data(Packet* packet, char* data, uint8_t data_size) {
    for (int i = 0; i < data_size; i++) {
      packet->RawData[i] = data[i];
    }
    packet->RawData[data_size] = 0x17;
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

  void send(float* data, int id) {
    // Really can't be bothered to figure out how floats work in binary tbh.
    Packet pack = this->create_packet(5, id);
    char str[29]; 
    dtostrf(*data, 2, 5, str);
    this->set_data(&pack, str, strlen(str));
    this->send_packet(&pack);
  }

  void send(char* data, int id) {
    Packet pack = this->create_packet(1, id);
    this->set_data(&pack, data, strlen(data));
    this->send_packet(&pack);
  }

  void send(int16_t* data, int id) {
    Packet pack;
    uint8_t data_to_send[2];
    if (*data < 0) {
      pack = this->create_packet(3,id);
    } else {
      pack = this->create_packet(2, id);
    }
    this->convert_u16(*data, data_to_send);
    this->set_data(&pack, data_to_send, sizeof(data_to_send) / sizeof(data_to_send[0]));
    this->send_packet(&pack);
  }
};

PacketHandler* packet_handler = new PacketHandler();
Packet pack;
void setup() {
  Serial.begin(9600);
  pinMode(13, OUTPUT);
  packet_handler->send("Connected!", 0);
}

void loop() {
  //packet_handler->send_packet(pack);
  int16_t t = -5325;
  int16_t t_2 = 500;
  int16_t t_3 = -500;
  float float_test = 4.432;
  //packet_handler->send(&t, 1);
  //packet_handler->send("4.432432", 3)
  packet_handler->send(&float_test, 1);
  delay(10);
  if (temp_bool) {
    packet_handler->send(&t_2, 2);
  } else {
    packet_handler->send(&t_3, 2);
  }
  if (temp_count % 5 == 0) {
    temp_bool = !temp_bool;
  }
  temp_count++;
  delay(1000);
}

// arduino-cli compile --fqbn arduino:avr:uno testSketch && arduino-cli upload testSketch -p /dev/ttyUSB0 -b arduino:avr:uno    
