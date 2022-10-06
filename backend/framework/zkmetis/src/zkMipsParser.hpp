#ifndef ZKMIPS_PARSER_HPP
#define ZKMIPS_PARSER_HPP

#include <iostream>
#include <string>
#include <vector>
#include <regex>
#include <fstream>
#include <jsoncpp/json/json.h>
#include <boost/algorithm/string.hpp>

#include "RAM/RAMInstance.hpp"

using namespace std;

string remove_extension(const string&);
string zkmips2ZkmetisInstruction(string&, string&, string&, string&);
string get_native_register(const string&);
string get_immediate_or_register(const string&);
void unroll_macros(vector<string>&, const string&);
string parse_zkmips(const string, const string, const string&, const bool);

#endif // ZKMIPS_PARSER_HPP
