#include "engine.h"
#include <stdio.h>
#include "predict.h"


/* functions prototype */
static void	ibus_eei_engine_class_init	(IBusEEIEngineClass	*klass);
static void	ibus_eei_engine_init		(IBusEEIEngine		*engine);
static void	ibus_eei_engine_destroy		(IBusEEIEngine		*engine);
static void ibus_eei_engine_enable      (IBusEngine             *engine);


G_DEFINE_TYPE (IBusEEIEngine, ibus_eei_engine, IBUS_TYPE_ENGINE)


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

    engine_class->process_key_event = ibus_eei_engine_process_key_event;
    engine_class->enable = ibus_eei_engine_enable;
}

static void
ibus_eei_engine_init (IBusEEIEngine *eei)
{
    eei->engine_core = new_engine_core(eei);
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


//static void
//ibus_eei_engine_update_preedit (IBusEEIEngine *eei)
//{
//    IBusText *text;
//
//    text = ibus_text_new_from_static_string (eei->preedit->str);
//    text->attrs = ibus_attr_list_new ();
//
//    ibus_attr_list_append (text->attrs,
//                           ibus_attr_underline_new (IBUS_ATTR_UNDERLINE_SINGLE, 0, eei->preedit->len));
//
//
//    ibus_engine_update_auxiliary_text((IBusEngine *)eei, ibus_text_new_from_static_string("AUXTEXT"), TRUE);
//    ibus_engine_update_preedit_text ((IBusEngine *)eei,
//                                     text,
//                                     eei->cursor_pos,
//                                     TRUE);
//
//}

///* commit preedit to client and update preedit */
//static gboolean
//ibus_eei_engine_commit_preedit (IBusEEIEngine *eei)
//{
//    if (eei->preedit->len == 0)
//        return FALSE;
//
//    ibus_eei_engine_commit_string (eei, eei->preedit->str);
//    g_string_assign (eei->preedit, "");
//    eei->cursor_pos = 0;
//
//    ibus_eei_engine_update (eei);
//
//    return TRUE;
//}

