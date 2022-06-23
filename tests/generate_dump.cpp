#include <stdio.h>
#include <iostream>
#include <fstream>
#include <string.h>
#include <sys/time.h>

using namespace std;

long javaTimeMillis() {
  timeval time;
  int status = gettimeofday(&time, NULL);
  return long(time.tv_sec) * 1000  +  long(time.tv_usec / 1000);
}

int main() {
    
    ofstream file("dump.hprof", ios::out | ios::binary);
    if(!file) {
        cout << "Cannot open file" << endl;
        return 1;
    }

    // Write hprof header
    const char* format_name = "JAVA PROFILE 1.0.1";
    file.write(format_name, (int)strlen(format_name));
    int zero_byte = 0;
    file.write((char*)&zero_byte, 1);
    int ptr_size = sizeof(void*);
    // jvm stores variables in big-endian
    ptr_size = __builtin_bswap32(ptr_size);
    file.write(reinterpret_cast<const char*>(&ptr_size), 4);
    long time_millis = javaTimeMillis();
    long time_millis_be = __builtin_bswap64(time_millis);
    file.write((char*)&time_millis_be, 8);

    // add a record
    // add record's header
    char tag = 0x01;
    file.write((char*)&tag, 1);
    int time_diff = javaTimeMillis() - time_millis;
    int time_diff_be = __builtin_bswap32(time_diff);
    file.write((char*)&time_diff_be, 4);
    const char* test_string = "test string";
    int len = 4 + strlen(test_string);
    len = __builtin_bswap32(len);
    file.write((char*)&len, 4);

    // write body
    int id = 1;
    id = __builtin_bswap32(id);
    file.write((char*)&id, 4);
    file.write(test_string, strlen(test_string));

    // Add HEAP DUMP record
    tag = 0x0C;
    file.write((char*)&tag, 1);
    time_diff = javaTimeMillis() - time_millis;
    time_diff_be = __builtin_bswap32(time_diff);
    file.write((char*)&time_diff_be, 4);
    len = 0;
    len = __builtin_bswap32(len);
    file.write((char*)&len, 4);

    // Add sub-tag
    tag = 0xFF;
    // file.write((char*)&tag, 1);
    // time_diff = javaTimeMillis() - time_millis;
    // time_diff_be = __builtin_bswap32(time_diff);
    // file.write((char*)&time_diff_be, 4);
    // len = 1;
    // len = __builtin_bswap32(len);
    // file.write((char*)&len, 4);

    // short sub_id = 2;
    // sub_id = __builtin_bswap16(sub_id);
    // file.write((char*)&sub_id, 2);

    file.close();
    if(!file.good()) {
        cout << "Error occured while writing" << endl;
        return 1;
    }

    return 0;
}