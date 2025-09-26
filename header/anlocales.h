#ifndef ANLOCALES_H
#define ANLOCALES_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>

// forward declarations
typedef struct AnLocales AnLocales;
typedef struct Locale Locale;

// ==================== AnLocales ====================
// create AnLocales with custom directories
AnLocales* anlocales_new_with_paths(const char* locales_path, const char* temp_path, const char* settings_file_path);

// create AnLocales with usual directories
AnLocales* anlocales_new();

// free AnLocales object returned from library
void anlocales_free(AnLocales* ptr);

// load locale
Locale* locale_load(AnLocales* al, const char* name);
void locale_free(Locale* loc);

// get default locale
Locale* anlocales_default_locale(AnLocales* al);

// get fallback locale
Locale* anlocales_fallback_locale(AnLocales* al);

// ==================== Locale ====================

// translation
const char* locale_t(Locale* loc, const char* key);

// date formatting
const char* locale_format_date(Locale* loc, int year, unsigned int month, unsigned int day);

// money formatting
const char* locale_format_money(Locale* loc, double amount);

// numeric formatting
const char* locale_format_numeric(Locale* loc, double number);

// string comparison (collation)
int locale_compare(Locale* loc, const char* a, const char* b);

// plural word (returns correct form of word for n)
const char* locale_plural_word(Locale* loc, const char* key, uint32_t n);

// free C strings returned from library
void locale_free_str(char* s);

#ifdef __cplusplus
}
#endif

#endif // ANLOCALES_H
