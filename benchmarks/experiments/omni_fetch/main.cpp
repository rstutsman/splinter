#include <limits>
#include <iostream>
#include <stdlib.h>
#include <string>
#include <chrono>

#define MAP_SIZE 3800000
#define RNG_SEED 20170404


/// SECTION: HASH MAP
struct BucketNode {
  bool valid;
  std::string key;
  double val;
  BucketNode* next;
};

class Map {
private:
  BucketNode* raw_map;
public:
  Map() {
    this->raw_map = (BucketNode*)calloc(MAP_SIZE * 2, sizeof(BucketNode));
  }
  ~Map() {
    free(this->raw_map);
  }

  void put(std::string key, double val) {
    size_t hash = std::hash<std::string>{}(key);
    
    // probe
    BucketNode* node = &this->raw_map[hash % MAP_SIZE];
    if (node->valid && node->key != key) {
      while (node->next != NULL && node->key != key) {
        node = node->next;
      }
      
      if (node->key != key) {
        node->next = (BucketNode*)malloc(sizeof(BucketNode));
        node = node->next;
      }
    }
    if (node->key == key) {
      return;
    }
    node->key = key;
    node->val = val;
    node->next = NULL;
    node->valid = true;
  }

  double hash_get(std::string key) {
    size_t hash = std::hash<std::string>{}(key);
    
    // lots of assumptions about correctness of input
    // and state of map made in the next few lines.
    BucketNode* node = &this->raw_map[hash % MAP_SIZE];
    
    if (node->key == key) {
      return node->val;
    } else {
      while (node->key != key) {
        node = node->next;
      }
      return node->val;
    }
  }
};
/// END SECTION

/// SECTION: RANDOM GENERATION
double next_double() {
  double f = (double)rand() / RAND_MAX;
  return f * 90.0;
}

std::string next_string() {
  static const char alphas[] = "abcdefghijklmnopqrstuvwxyz";
  char buffer[255];
  for (int i = 0; i < 255; i++) {
    buffer[i] = alphas[rand() % 26];
  }
  return std::string(buffer, 255);
}
/// END SECTION

int main(int argc, char** argv) {
  // read args
  int selectivity = atoi(argv[1]);
  int buffer_size = atoi(argv[2]);

  // build map
  Map map;

  // build strings to get array
  std::string *strings = (std::string*)malloc((MAP_SIZE * 2 / selectivity) * sizeof(std::string));
  int strings_csr = 0;

  // populate map
  srand(RNG_SEED);
  for (int i = 0; i < MAP_SIZE; i++) {
    std::string key = next_string();
    double val = next_double();
    if (rand() % selectivity == 0) {
      strings[strings_csr++] = key;
    }
    map.put(key, val);
  }

  // calculate average
  int stride = 0, num_strings = strings_csr;
  double* buffer = (double*)malloc(buffer_size * sizeof(double));
  double sum = 0;
  
  auto begin = std::chrono::high_resolution_clock::now();
  for (; stride < num_strings; stride += buffer_size) {
    
    // this is the simulation of prefetch
    for (int i = 0; i < buffer_size && i + stride < num_strings; i++) {
      buffer[i] = map.hash_get(strings[stride + i]);
    }

    // this is the real work
    for (int i = 0; i < buffer_size; i++) {
      sum += buffer[i];
    }
  }
  auto end = std::chrono::high_resolution_clock::now();
  
  auto durr = std::chrono::duration_cast<std::chrono::milliseconds>(end-begin).count();
  
  std::cout << "{\n"\
    << "\t\"sum\":" << sum << ",\n"\
    << "\t\"average\":" << sum / num_strings << ",\n"\
    << "\t\"duration\":" << durr << ",\n"\
    << "\t\"nStrings\":" << num_strings << ",\n"\
    << "\t\"bufferSize\":" << buffer_size << ",\n"\
    << "}\n";
}


