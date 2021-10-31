#include "engine.h"
#include <stdio.h>
#include "predict.h"


/* functions prototype */
static void	ibus_eei_engine_class_init	(IBusEEIEngineClass	*klass);
static void	ibus_eei_engine_init		(IBusEEIEngine		*engine);
static void	ibus_eei_engine_destroy		(IBusEEIEngine		*engine);
static gboolean 
			ibus_eei_engine_process_key_event
                                            (IBusEngine             *engine,
                                             guint               	 keyval,
                                             guint               	 keycode,
                                             guint               	 modifiers);
static void ibus_eei_engine_enable      (IBusEngine             *engine);


static void ibus_eei_engine_commit_string
                                            (IBusEEIEngine      *eei,
                                             const gchar            *string);
static void ibus_eei_engine_update      (IBusEEIEngine      *eei);


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
    eei->preedit = g_string_new ("");
    eei->cursor_pos = 0;
    eei->lookup_table_visible = FALSE;

    eei->engine_state = new_engine_state();
    eei->table = ibus_lookup_table_new (9, 0, TRUE, TRUE);
    g_object_ref_sink (eei->table);
}

static void
ibus_eei_engine_destroy (IBusEEIEngine *eei)
{
    if (eei->preedit) {
        g_string_free (eei->preedit, TRUE);
        eei->preedit = NULL;
    }

    if (eei->table) {
        g_object_unref (eei->table);
        eei->table = NULL;
    }

    if (eei->engine_state) {
        free_engine_state(eei->engine_state);
    }

	((IBusObjectClass *) ibus_eei_engine_parent_class)->destroy ((IBusObject *)eei);
}

static void
ibus_eei_engine_enable  (IBusEngine *engine)
{
    // dummy call to tell the input context that the engine will utilize surrounding-text
    ibus_engine_get_surrounding_text (engine, NULL, NULL, NULL);
}

static void
ibus_eei_engine_update_lookup_table (IBusEEIEngine *eei)
{
    WordPredictions predictions = get_word_predictions(eei->preedit->str);
    gint i;

    if (eei->preedit->len == 0) {
        ibus_eei_engine_hide_lookup_table(eei);
        return;
    }

    ibus_lookup_table_clear (eei->table);

    if (predictions.len == 0) {
        ibus_eei_engine_hide_lookup_table(eei);
        return;
    }

    for (i = 0; i < predictions.len; i++) {
        ibus_lookup_table_append_candidate (eei->table, ibus_text_new_from_string (predictions.words[i]));
    }

    ibus_engine_update_lookup_table ((IBusEngine *) eei, eei->table, TRUE);
    eei->lookup_table_visible = TRUE;
    free_word_predictions(predictions);
}

static void
ibus_eei_engine_update_preedit (IBusEEIEngine *eei)
{
    IBusText *text;

    text = ibus_text_new_from_static_string (eei->preedit->str);
    text->attrs = ibus_attr_list_new ();
    
    ibus_attr_list_append (text->attrs,
                           ibus_attr_underline_new (IBUS_ATTR_UNDERLINE_SINGLE, 0, eei->preedit->len));


    ibus_engine_update_auxiliary_text((IBusEngine *)eei, ibus_text_new_from_static_string("AUXTEXT"), TRUE);
    ibus_engine_update_preedit_text ((IBusEngine *)eei,
                                     text,
                                     eei->cursor_pos,
                                     TRUE);

}

/* commit preedit to client and update preedit */
static gboolean
ibus_eei_engine_commit_preedit (IBusEEIEngine *eei)
{
    if (eei->preedit->len == 0)
        return FALSE;
    
    ibus_eei_engine_commit_string (eei, eei->preedit->str);
    g_string_assign (eei->preedit, "");
    eei->cursor_pos = 0;

    ibus_eei_engine_update (eei);

    return TRUE;
}

static gboolean
ibus_eei_engine_commit_word (IBusEEIEngine *eei)
{
    guint cursor_pos = ibus_lookup_table_get_cursor_pos(eei->table);
    IBusText *text = ibus_lookup_table_get_candidate(eei->table, cursor_pos);
    ibus_engine_commit_text((IBusEngine *)eei, text);
    g_string_assign(eei->preedit, "");
    eei->cursor_pos = 0;

    ibus_eei_engine_update(eei);

    return TRUE;
}


static void
ibus_eei_engine_commit_string (IBusEEIEngine *eei,
                                   const gchar       *string)
{
    IBusText *text;
    text = ibus_text_new_from_static_string (string);
    ibus_engine_commit_text ((IBusEngine *)eei, text);
}

static void
ibus_eei_engine_update (IBusEEIEngine *eei)
{
    ibus_eei_engine_update_preedit (eei);
    ibus_eei_engine_hide_lookup_table(eei);
}

#define is_printable(c) ((c) >= IBUS_space && (c) <=IBUS_asciitilde)

static gboolean 
ibus_eei_engine_process_key_event (IBusEngine *engine,
                                       guint       keyval,
                                       guint       keycode,
                                       guint       modifiers)
{
    IBusText *text;
    IBusEEIEngine *eei = (IBusEEIEngine *)engine;

    if (modifiers & IBUS_RELEASE_MASK) //if it's a key release
        return FALSE;

    modifiers &= (IBUS_CONTROL_MASK | IBUS_MOD1_MASK);

    if (modifiers == IBUS_CONTROL_MASK && keyval == IBUS_s) { //update lookup table if CTRL S
        ibus_eei_engine_update_lookup_table (eei);
        return TRUE;
    }

    if (modifiers != 0) {
        if (eei->preedit->len == 0)
            return FALSE;
        else
            return TRUE;
    }


    switch (keyval) {
    case IBUS_space:
        g_string_append (eei->preedit, " ");
        return ibus_eei_engine_commit_preedit (eei);
    case IBUS_Return:
        if (eei->lookup_table_visible) {
            return ibus_eei_engine_commit_word(eei);
        } else {
            return ibus_eei_engine_commit_preedit (eei);
        }

    case IBUS_Escape:
        if (eei->preedit->len == 0)
            return FALSE;

        g_string_assign (eei->preedit, "");
        eei->cursor_pos = 0;
        ibus_eei_engine_update (eei);
        return TRUE;        

    case IBUS_Left:
        if (eei->preedit->len == 0)
            return FALSE;
        if (eei->cursor_pos > 0) {
            eei->cursor_pos --;
            ibus_eei_engine_update (eei);
        }
        return TRUE;

    case IBUS_Right:
        if (eei->preedit->len == 0)
            return FALSE;
        if (eei->cursor_pos < eei->preedit->len) {
            eei->cursor_pos ++;
            ibus_eei_engine_update (eei);
        }
        return TRUE;
    
    case IBUS_Up:
        if (eei->preedit->len == 0)
            return FALSE;

        ibus_lookup_table_cursor_up(eei->table);
        ibus_engine_update_lookup_table ((IBusEngine *) eei, eei->table, TRUE);
        return TRUE;

    case IBUS_Down:
        if (eei->preedit->len == 0)
            return FALSE;


        ibus_lookup_table_cursor_down(eei->table);
        ibus_engine_update_lookup_table ((IBusEngine *) eei, eei->table, TRUE);

        return TRUE;

    case IBUS_BackSpace:
        if (eei->preedit->len == 0)
            return FALSE;
        if (eei->cursor_pos > 0) {
            eei->cursor_pos --;
            g_string_erase (eei->preedit, eei->cursor_pos, 1);
            ibus_eei_engine_update (eei);
        }
        return TRUE;
    
    case IBUS_Delete:
        if (eei->preedit->len == 0)
            return FALSE;
        if (eei->cursor_pos < eei->preedit->len) {
            g_string_erase (eei->preedit, eei->cursor_pos, 1);
            ibus_eei_engine_update (eei);
        }
        return TRUE;
    }

    if (is_printable (keyval)) {
        g_string_insert_c (eei->preedit,
                           eei->cursor_pos,
                           keyval);

        eei->cursor_pos ++;
        ibus_eei_engine_update (eei);
        
        return TRUE;
    }

    return FALSE;
}
