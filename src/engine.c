#include "engine.h"

typedef struct _IBusEEIEngine IBusEEIEngine;
typedef struct _IBusEEIEngineClass IBusEEIEngineClass;

struct _IBusEEIEngine {
	IBusEngine parent;

    /* members */
    GString *preedit;
    gint cursor_pos;

    IBusLookupTable *table;
};

struct _IBusEEIEngineClass {
	IBusEngineClass parent;
};

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
static void ibus_eei_engine_focus_in    (IBusEngine             *engine);
static void ibus_eei_engine_focus_out   (IBusEngine             *engine);
static void ibus_eei_engine_reset       (IBusEngine             *engine);
static void ibus_eei_engine_enable      (IBusEngine             *engine);
static void ibus_eei_engine_disable     (IBusEngine             *engine);
static void ibus_engine_set_cursor_location (IBusEngine             *engine,
                                             gint                    x,
                                             gint                    y,
                                             gint                    w,
                                             gint                    h);
static void ibus_eei_engine_set_capabilities
                                            (IBusEngine             *engine,
                                             guint                   caps);
static void ibus_eei_engine_page_up     (IBusEngine             *engine);
static void ibus_eei_engine_page_down   (IBusEngine             *engine);
static void ibus_eei_engine_cursor_up   (IBusEngine             *engine);
static void ibus_eei_engine_cursor_down (IBusEngine             *engine);
static void ibus_eei_property_activate  (IBusEngine             *engine,
                                             const gchar            *prop_name,
                                             gint                    prop_state);
static void ibus_eei_engine_property_show
											(IBusEngine             *engine,
                                             const gchar            *prop_name);
static void ibus_eei_engine_property_hide
											(IBusEngine             *engine,
                                             const gchar            *prop_name);

static void ibus_eei_engine_commit_string
                                            (IBusEEIEngine      *eei,
                                             const gchar            *string);
static void ibus_eei_engine_update      (IBusEEIEngine      *eei);


G_DEFINE_TYPE (IBusEEIEngine, ibus_eei_engine, IBUS_TYPE_ENGINE)

static void
ibus_eei_engine_class_init (IBusEEIEngineClass *klass)
{
	IBusObjectClass *ibus_object_class = IBUS_OBJECT_CLASS (klass);
	IBusEngineClass *engine_class = IBUS_ENGINE_CLASS (klass);
	
	ibus_object_class->destroy = (IBusObjectDestroyFunc) ibus_eei_engine_destroy;

    engine_class->process_key_event = ibus_eei_engine_process_key_event;
}

static void
ibus_eei_engine_init (IBusEEIEngine *eei)
{
    eei->preedit = g_string_new ("");
    eei->cursor_pos = 0;

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

	((IBusObjectClass *) ibus_eei_engine_parent_class)->destroy ((IBusObject *)eei);
}

static void
ibus_eei_engine_update_lookup_table (IBusEEIEngine *eei)
{
    gchar* sugs[] = {"foo", "bar"};
    gint n_sug, i;
    gboolean retval;

    if (eei->preedit->len == 0) {
        ibus_engine_hide_lookup_table ((IBusEngine *) eei);
        return;
    }

    ibus_lookup_table_clear (eei->table);
    n_sug = 2;

    if (n_sug == 0) {
        ibus_engine_hide_lookup_table ((IBusEngine *) eei);
        return;
    }

    for (i = 0; i < n_sug; i++) {
        ibus_lookup_table_append_candidate (eei->table, ibus_text_new_from_string (sugs[i]));
    }

    ibus_engine_update_lookup_table ((IBusEngine *) eei, eei->table, TRUE);

//    if (sugs)
//        eei_dict_free_suggestions (dict, sugs);
}

static void
ibus_eei_engine_update_preedit (IBusEEIEngine *eei)
{
    IBusText *text;
    gint retval;

    text = ibus_text_new_from_static_string (eei->preedit->str);
    text->attrs = ibus_attr_list_new ();
    
    ibus_attr_list_append (text->attrs,
                           ibus_attr_underline_new (IBUS_ATTR_UNDERLINE_SINGLE, 0, eei->preedit->len));

    if (eei->preedit->len > 0) {
        retval = 2;
        if (retval != 0) {
            ibus_attr_list_append (text->attrs,
                               ibus_attr_foreground_new (0xff0000, 0, eei->preedit->len));
        }
    }

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
    ibus_engine_hide_lookup_table ((IBusEngine *)eei);
}

#define is_alpha(c) (((c) >= IBUS_a && (c) <= IBUS_z) || ((c) >= IBUS_A && (c) <= IBUS_Z))

static gboolean 
ibus_eei_engine_process_key_event (IBusEngine *engine,
                                       guint       keyval,
                                       guint       keycode,
                                       guint       modifiers)
{
    IBusText *text;
    IBusEEIEngine *eei = (IBusEEIEngine *)engine;

    if (modifiers & IBUS_RELEASE_MASK)
        return FALSE;

    modifiers &= (IBUS_CONTROL_MASK | IBUS_MOD1_MASK);

    if (modifiers == IBUS_CONTROL_MASK && keyval == IBUS_s) {
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
        return ibus_eei_engine_commit_preedit (eei);

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
        if (eei->cursor_pos != 0) {
            eei->cursor_pos = 0;
            ibus_eei_engine_update (eei);
        }
        return TRUE;

    case IBUS_Down:
        if (eei->preedit->len == 0)
            return FALSE;
        
        if (eei->cursor_pos != eei->preedit->len) {
            eei->cursor_pos = eei->preedit->len;
            ibus_eei_engine_update (eei);
        }
        
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

    if (is_alpha (keyval)) {
        g_string_insert_c (eei->preedit,
                           eei->cursor_pos,
                           keyval);

        eei->cursor_pos ++;
        ibus_eei_engine_update (eei);
        
        return TRUE;
    }

    return FALSE;
}
