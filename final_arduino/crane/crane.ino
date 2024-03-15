#include <Adafruit_SSD1306.h> //display libraries
#include <Adafruit_GFX.h>
#include <Adafruit_SH110X.h>
#include <Wire.h> //Interfaces for the display
#include <SPI.h>
#include <ArduinoBLE.h> // BLE Library

#define SW1 11
#define SW2 12
#define SW3 13

#define RADIUS 2

#define OPTICAL_PIN 2

#define MOTOR_IN1 3
#define MOTOR_IN2 4
#define MOTOR_EN 5

#define LOAD_CELL_PIN_IN A2

//
// BLE 
//

//Create the Service and give at name and a uid
BLEService AinService("4315b8fb-7cca-4ba6-a4c0-c3c0c915180f"); //This is a custom 128 bit service.It must be unique code, 

// Characterisics aka setting up data transfer
// All of the following have the following options: Write, notify changes, read, and broadcast 
BLEIntCharacteristic Rev("4315b8fb-7cca-4ba6-a4c0-c3c0c915180f", BLEWrite | BLENotify | BLERead | BLEBroadcast); 
BLEFloatCharacteristic Newton("c3ccbb8e-930c-4add-b57a-ce692b0c36ae", BLEWrite | BLENotify| BLERead | BLEBroadcast);
BLEFloatCharacteristic Speed("a3f5051e-d355-4dc6-bcb2-ed79a68d15a5", BLEWrite | BLENotify| BLERead | BLEBroadcast);
BLEFloatCharacteristic Rpm("d1e76dde-62ea-4073-ab8a-005ff0536f63", BLEWrite | BLENotify| BLERead | BLEBroadcast);

uint16_t oldRev = 0;  // last reading of revolutions 

int load_cell_in = 0; 
float result = 0;

uint8_t motor_speed_out = 127;

//
// BLE
//

Adafruit_SH1107 display = Adafruit_SH1107(64, 128, &Wire); //setting up the display
const int switch1 = 11; //SW1 on D11 is called switch1
const int pwmout = 5; //D5 used for PWM and called pwmout
int check_switch1, check_switch2, check_switch3; //used to see if SW1 is pressed
int count;
volatile bool colourOne;  // detect colour change
volatile int16_t currentRev = 0;

float rpm = 0;

bool isClock; // Clockwise ?
bool isStopped; 

void setup() {
  Serial.begin(9600);
  delay(250); // wait for the OLED to power up
  display.begin(0x3C, true); // Address 0x3C default
  delay(1000); // delay to allow display to settle
  display.setRotation(3); //values 1 - 4
  display.setTextSize(2); // from 1 to 8 [pixel 6x8 for size=1] with 1 pixel spacing
  display.setTextColor(WHITE, BLACK);
  display.clearDisplay();
  display.setCursor(0,0);
  display.println("Connecting BT");
  display.display();
  pinMode(MOTOR_IN1, OUTPUT); //setting a digital pin as an output
  pinMode(MOTOR_IN2, OUTPUT);
  pinMode(MOTOR_EN, OUTPUT);
  pinMode(SW1, INPUT);
  pinMode(SW2, INPUT);
  pinMode(SW3, INPUT);
  attachInterrupt(digitalPinToInterrupt(OPTICAL_PIN), opticalInterrupt, FALLING); 

  //
  // BLE
  //

   if (!BLE.begin()) 
  {
    Serial.println("starting BLE failed!");
    while (1);
  }

  //printing the sender's information
  Serial.print("Sender's mac is  ");
  Serial.println(BLE.address()); //using a BLE function
  BLE.setLocalName("AinMonitor"); //just a name
  BLE.setAdvertisedService(AinService); //matches earlier
  AinService.addCharacteristic(Rev); // this is 19B10001-E8F2-537E-4F6C-D104768A1214
  AinService.addCharacteristic(Newton);
  AinService.addCharacteristic(Rpm);
  AinService.addCharacteristic(Speed);
  BLE.addService(AinService); // Add the Ain service to BLE
  Rev.writeValue(oldRev); // set initial value for this characteristic. We can write to the characteristic or read from it.
  Newton.writeValue(0.0);
  Rpm.writeValue(0.0);
  Speed.writeValue(0.0);
  // start advertising BLE
  BLE.advertise();
  Serial.println("BLE peripheral advertising Ainmonitor...");

  Rev.broadcast();//broadcast the charcteristic called VinLevel

  //
  //BLE
  //

}

void calc_revs_per_minute() {
  uint16_t tmp = millis();
  tmp = (tmp / 1000)/60; // Convert to minutes to make it easier
  rpm = (currentRev / 2.0) / tmp;
  Rpm.writeValue(rpm);
}

float calc_speed() {
  float tmp = (2 * PI * RADIUS) / rpm;
  Speed.writeValue(tmp);
  return tmp; 
}

void opticalInterrupt() {
  if (isStopped) return; // This shouldn't really be needed 
  if (isClock) {
    currentRev++;
  } else {
    currentRev--;
  }
}

void loop() {
  Serial.println(digitalRead(OPTICAL_PIN));
  
  // wait for a BLE central if a remote device is found its called central
  BLEDevice central = BLE.central();
 
  // if a central is connected to the peripheral:
  if (central) { 
    Serial.print("Connected to central: ");
    // print the central's BT address:
    Serial.println(central.address());
    
    while (central.connected()) 
    {  
      check_switch1 = digitalRead(SW1); // check for switch being pressed
      check_switch2 = digitalRead(SW2); // check for switch being pressed
      check_switch3 = digitalRead(SW3); // check for switch being pressed
      
      //Serial.println("----------------");
      if (check_switch1 == 0) 
      {
        //generate_pwm(); //run a function we have written called generate_pwm
        stopMotor();
      } 
      else if (check_switch3 == 0) {
          setAntiClockwise();
          //currentRev--;
      }
      else if (check_switch2 == 0) {
          setClockwise();
          //currentRev++;
      }
      calc_revs_per_minute();
      displayHandler();
      readValue();
      updateRev(); // Check if revolution changed then send the updated signal to the receiver
    }

    Serial.print("Disconnected from central: ");
    Serial.println(central.address());
  } 

}

void displayHandler() {
  display.clearDisplay();
  display.setCursor(0, 0);
  if (isStopped) {
    display.println("Stopped");
    digitalWrite(8, LOW);
  }
  else if (isClock) {
    display.println("Clockwise");
    digitalWrite(8, HIGH);
  } else if (!isClock) {
    display.println("Anti-CW");
    digitalWrite(8, LOW);
  }
  char revStr[64];

  // Remove this for speed!
  //snprintf(revStr, 64, "%d rev | %fm/s", currentRev / 2, calc_speed());

  snprintf(revStr, 64, "Rev: %d", currentRev / 2);
  display.print(revStr);
  snprintf(revStr, 64, "Force: %dN", result);
  display.display();
}

// CLOCKWISE 
// IN1: LOW IN2: HIGH

void setClockwise() {
  isStopped = false;
  analogWrite(MOTOR_EN, motor_speed_out);
  digitalWrite(MOTOR_IN1, LOW);
  digitalWrite(MOTOR_IN2, HIGH);
  isClock = true;
}

// ANTI
// IN1: HIGH IN2: LOW
void setAntiClockwise() {
  isStopped = false;
  analogWrite(MOTOR_EN, motor_speed_out);
  digitalWrite(MOTOR_IN2, LOW);
  digitalWrite(MOTOR_IN1, HIGH);
  isClock = false;
}

// IN1: LOW IN2: LOW
void stopMotor() {
  digitalWrite(MOTOR_IN1, LOW);
  digitalWrite(MOTOR_IN2, LOW);
  analogWrite(MOTOR_EN, LOW);
  isStopped = true;
}

// Really not sure how to even implement this considering we are already using all three buttons
// I imagine it'd either be a button combination, reading the built-in potentiometer, or just pressing the button multiple times
// I really don't know which is best
void adjust_speed(boolean is_increase, uint8_t increment) {
  if (is_increase && (int) motor_speed_out + (int) increment <= 255) { // Overflowing probably isn't a good idea
    motor_speed_out += increment;
  } else if (!is_increase && (int) motor_speed_out - (int) increment >= 0){ // Maybe we shouldn't underflow it either
    motor_speed_out -= increment;
  } else if (is_increase && (int) motor_speed_out + (int) increment > 255) { // Clamp it
    motor_speed_out = 255;
  } else if (!is_increase && (int) motor_speed_out - (int) < 0) {
    motor_speed_out = 0;
  }
}

void readValue() {
  load_cell_in = analogRead(LOAD_CELL_PIN_IN);
  result = load_cell_in * (3.3 / 1024.0);
  float magic_number = 39.24 / 2.0; // Theoretically the amount of Newtons per input voltage
  result = result * magic_number;
  Serial.println(result);
  Newton.writeValue(result);
}

void updateRev() 
{
  if (currentRev != oldRev) {// Revolution has changed
    Rev.writeValue(currentRev / 2); 
    oldRev = currentRev;    // Save current value for future comparison 
  }
}
