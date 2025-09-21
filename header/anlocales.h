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
AnLocales* anlocales_new();
void anlocales_free(AnLocales* ptr);

// load locale
Locale* locale_load(AnLocales* al, const char* name);
void locale_free(Locale* loc);

// get default/fallback locale
Locale* anlocales_default_locale(AnLocales* al);
Locale* anlocales_fallback_locale(AnLocales* al);

// ==================== Locale ====================

// translation
const char* locale_t(Locale* loc, const char* key);

// date formatting
const char* locale_format_date(Locale* loc, int year, unsigned int month, unsigned int day);

// money formatting
const char* locale_format_money(Locale* loc, double amount);

// string comparison (collation)
int locale_compare(Locale* loc, const char* a, const char* b);

// plural check
bool locale_plural(Locale* loc, uint32_t n);

// free C strings returned from library
void locale_free_str(char* s);

#ifdef __cplusplus
}
#endif

#endif // ANLOCALES_H
