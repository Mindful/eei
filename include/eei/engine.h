#ifndef __ENGINE_H__
#define __ENGINE_H__

#include <ibus.h>

typedef struct _IBusEEIEngine IBusEEIEngine;
typedef struct _IBusEEIEngineClass IBusEEIEngineClass;
typedef struct EngineCore EngineCore;

static gboolean GBOOL_FALSE = FALSE;
static gboolean GBOOL_TRUE = TRUE;

struct _IBusEEIEngine {
    IBusEngine parent;
    IBusLookupTable *table;

    EngineCore *engine_core;
};

struct _IBusEEIEngineClass {
    IBusEngineClass parent;
};

#define IBUS_TYPE_EEI_ENGINE	\
	(ibus_eei_engine_get_type ())

GType   ibus_eei_engine_get_type    (void);

#endif
