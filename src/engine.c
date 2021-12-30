#include "engine.h"
#include <stdio.h>
#include "predict.h"


/* functions prototype */
static void	ibus_eei_engine_class_init	(IBusEEIEngineClass	*klass);
static void	ibus_eei_engine_init		(IBusEEIEngine		*engine);
static void	ibus_eei_engine_destroy		(IBusEEIEngine		*engine);
static void ibus_eei_engine_enable      (IBusEngine             *engine);


G_DEFINE_TYPE (IBusEEIEngine, ibus_eei_engine, IBUS_TYPE_ENGINE)

static IBusEngineClass* parent_class = NULL;

static gboolean can_get_surrounding_text(IBusEEIEngine *eei)
{
    return eei->parent.client_capabilities & IBUS_CAP_SURROUNDING_TEXT;
}

static void
ibus_eei_engine_class_init (IBusEEIEngineClass *klass)
{
	IBusObjectClass *ibus_object_class = IBUS_OBJECT_CLASS (klass);
	IBusEngineClass *engine_class = IBUS_ENGINE_CLASS (klass);

	ibus_object_class->destroy = (IBusObjectDestroyFunc) ibus_eei_engine_destroy;
    parent_class = (IBusEngineClass* )g_type_class_peek_parent(klass);

    engine_class->process_key_event = ibus_eei_engine_process_key_event;
    engine_class->page_down = ibus_eei_engine_page_down_button;
    engine_class->page_up = ibus_eei_engine_page_up_button;
    engine_class->candidate_clicked = ibus_eei_engine_candidate_clicked;
    engine_class->focus_out = ibus_eei_engine_focus_out;
    engine_class->enable = ibus_eei_engine_enable;
}

static void
ibus_eei_engine_init (IBusEEIEngine *eei)
{
    eei->engine_core = new_engine_core(eei, parent_class);
    eei->table = ibus_lookup_table_new (9, 0, TRUE, TRUE);
    g_object_ref_sink (eei->table);
}

static void
ibus_eei_engine_destroy (IBusEEIEngine *eei)
{
    if (eei->table) {
        g_object_unref (eei->table);
        eei->table = NULL;
    }

    if (eei->engine_core) {
        free_engine_core(eei->engine_core);
        eei->engine_core = NULL;
    }

	((IBusObjectClass *) ibus_eei_engine_parent_class)->destroy ((IBusObject *)eei);
}

static void
ibus_eei_engine_enable  (IBusEngine *engine)
{
    // dummy call to tell the input context that the engine will utilize surrounding-text
    ibus_engine_get_surrounding_text (engine, NULL, NULL, NULL);
}

