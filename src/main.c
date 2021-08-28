/* vim:set et sts=4: */

#include <ibus.h>
#include <stdio.h>
#include "engine.h"

static IBusBus *bus = NULL;
static IBusFactory *factory = NULL;

/* command line options */
static gboolean ibus = FALSE;
static gboolean verbose = FALSE;

static const GOptionEntry entries[] =
{
    { "ibus", 'i', 0, G_OPTION_ARG_NONE, &ibus, "component is executed by ibus", NULL },
    { "verbose", 'v', 0, G_OPTION_ARG_NONE, &verbose, "verbose", NULL },
    { NULL },
};

static void
ibus_disconnected_cb (IBusBus  *bus,
                      gpointer  user_data)
{
    ibus_quit ();
}

static void debug_print(char* text) {
    printf("%s\n", text);
    fflush(stdout);
}


static void
init (void)
{
    ibus_init ();

    bus = ibus_bus_new ();
    g_object_ref_sink (bus);
    g_signal_connect (bus, "disconnected", G_CALLBACK (ibus_disconnected_cb), NULL);
	
    factory = ibus_factory_new (ibus_bus_get_connection (bus));
    g_object_ref_sink (factory);
    ibus_factory_add_engine (factory, "eei", IBUS_TYPE_EEI_ENGINE);

    if (ibus) {
        debug_print("requesting name");
        ibus_bus_request_name (bus, "joshua.tanner.IBus.eei", 0); //hangs if name doesn't match eei.xml
    }
    else {
        IBusComponent *component;

        component = ibus_component_new ("joshua.tanner.IBus.eei",
                                        "Extended English Input",
                                        "0.1.0",
                                        "GPL",
                                        "Joshua Tanner <mindful.jt@gmail.com>",
                                        "http://code.google.com/p/ibus/",
                                        "",
                                        "ibus-eei");
        debug_print("adding engine");
        ibus_component_add_engine (component,
                                   ibus_engine_desc_new ("eei",
                                                         "Extended English Input",
                                                         "Extended English Input",
                                                         "en",
                                                         "GPL",
                                                         "Joshua Tanner <mindful.jt@gmail.com>",
                                                         PKGDATADIR"/icons/ibus-enchant.svg",
                                                         "us"));
        debug_print("registering component");
        ibus_bus_register_component (bus, component);
    }
}

int main(int argc, char **argv)
{
    debug_print("start main");

    GError *error = NULL;
    GOptionContext *context;

    /* Parse the command line */
    context = g_option_context_new ("- ibus template engine");
    g_option_context_add_main_entries (context, entries, "ibus-tmpl");

    if (!g_option_context_parse (context, &argc, &argv, &error)) {
      g_print ("Option parsing failed: %s\n", error->message);
      g_error_free (error);
      return (-1);
    }

    debug_print("halfway through main");

    /* Go */
    init ();
    ibus_main ();
    debug_print("done");
}
