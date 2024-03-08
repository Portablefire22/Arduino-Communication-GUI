#include <Adafruit_SSD1306.h> //display libraries
#include <Adafruit_GFX.h>
#include <Adafruit_SH110X.h>
#include <Wire.h> //Interfaces for the display
#include <SPI.h>
#include <ArduinoBLE.h> // BLE Library

#define SW1 11
#define SW2 12
#define SW3 13

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
//and new codes can be obtained from
//https://www.guidgenerator.com/online-guid-generator.aspx
//Add the characteristic - the example is a floating point number - give it a name
BLEIntCharacteristic Rev("4315b8fb-7cca-4ba6-a4c0-c3c0c915180f", BLEWrite | BLENotify | BLERead | BLEBroadcast); 
BLEFloatCharacteristic Newton("c3ccbb8e-930c-4add-b57a-ce692b0c36ae", BLEWrite | BLENotify| BLERead | BLEBroadcast);
//we can write, notify changes, read and broadcast the characteristic

uint16_t oldRev = 0;  // last V reading from analog input

int potenitometerValue = 0;
float result = 0;

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
  BLE.addService(AinService); // Add the Ain service to BLE
  Rev.writeValue(oldRev); // set initial value for this characteristic. We can write to the characteristic or read from it.

  // start advertising BLE
  BLE.advertise();
  Serial.println("BLE peripheral advertising Ainmonitor...");

  Rev.broadcast();//broadcast the charcteristic called VinLevel

  //
  //BLE
  //

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
  snprintf(revStr, 64, "Rev: %d", currentRev / 2);
  display.print(revStr);
  snprintf(revStr, 64, "Force: %dN", result);
  display.display();
}

// CLOCKWISE 
// IN1: LOW IN2: HIGH

void setClockwise() {
  isStopped = false;
  analogWrite(MOTOR_EN, 127);
  digitalWrite(MOTOR_IN1, LOW);
  digitalWrite(MOTOR_IN2, HIGH);
  isClock = true;
}

// ANTI
// IN1: HIGH IN2: LOW
void setAntiClockwise() {
  isStopped = false;
  analogWrite(MOTOR_EN, 127);
  digitalWrite(MOTOR_IN2, LOW);
  digitalWrite(MOTOR_IN1, HIGH);
  isClock = false;
}

// IN1: LOW IN2: LOW
void stopMotor() {
  digitalWrite(MOTOR_IN1, LOW);
  digitalWrite(MOTOR_IN2, LOW);
  isStopped = true;
}

void readValue() {
  potenitometerValue = analogRead(LOAD_CELL_PIN_IN);
  result = potenitometerValue * (3.3 / 1024.0);
  float magic_number = 39.24 / 2.0;
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
