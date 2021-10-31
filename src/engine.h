#ifndef __ENGINE_H__
#define __ENGINE_H__

#include <ibus.h>

typedef struct _IBusEEIEngine IBusEEIEngine;
typedef struct _IBusEEIEngineClass IBusEEIEngineClass;

struct _IBusEEIEngine {
    IBusEngine parent;

    /* members */
    GString *preedit;
    gint cursor_pos;

    IBusLookupTable *table;
    gboolean lookup_table_visible;
};

struct _IBusEEIEngineClass {
    IBusEngineClass parent;
};

#define IBUS_TYPE_EEI_ENGINE	\
	(ibus_eei_engine_get_type ())

GType   ibus_eei_engine_get_type    (void);

#endif
