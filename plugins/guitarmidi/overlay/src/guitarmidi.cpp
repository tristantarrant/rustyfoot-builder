/* GuitarMidi-LV2 Library
 * Copyright (C) 2022 Gerald Mwangi
 *
 * This program is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General
 * Public License along with this program; if not, write to the
 * Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA  02110-1301  USA
 */

/** Include standard C headers */
#include <cassert>
#include <cstring>
#include <math.h>
#include <stdlib.h>
#include <lv2/core/lv2.h>
#include <fretboard.hpp>

#define AMP_URI "http://github.com/geraldmwangi/GuitarMidi-LV2"
#define PROCESS_BLOCK_SIZE 256

typedef struct {
	FretBoard *fretboard;
	const float *host_input;
	float buffer[PROCESS_BLOCK_SIZE];
	uint32_t buffer_pos;
} GuitarMidiPlugin;

static LV2_Handle
instantiate(const LV2_Descriptor *descriptor,
			double rate,
			const char *bundle_path,
			const LV2_Feature *const *features)
{

	LV2_URID_Map *map = NULL;
	LV2_Log_Log *log = NULL;
	printf("Loading plugin\n");
	for (int i = 0; features[i]; ++i)
	{
		if (!strcmp(features[i]->URI, LV2_URID__map))
		{
			map = (LV2_URID_Map *)features[i]->data;
		}
		else if (!strcmp(features[i]->URI, LV2_LOG__log))
		{
			log = (LV2_Log_Log *)features[i]->data;
		}
	}
	lv2_log_logger_init(&g_logger, map, log);
	if (!map)
	{
		lv2_log_error(&g_logger, "Host does not support urid:map\n");
		return NULL;
	}

	GuitarMidiPlugin *plugin = new GuitarMidiPlugin();
	plugin->host_input = NULL;
	plugin->buffer_pos = 0;
	memset(plugin->buffer, 0, sizeof(plugin->buffer));

	plugin->fretboard = new FretBoard(map, rate);
	if (!plugin->fretboard->initialize(std::string(bundle_path)))
	{
		lv2_log_error(&g_logger, "Failed to initialize FretBoard\n");
		delete plugin->fretboard;
		delete plugin;
		return NULL;
	}
	return (LV2_Handle)plugin;
}

static void
connect_port(LV2_Handle instance,
			 uint32_t port,
			 void *data)
{
	GuitarMidiPlugin *plugin = (GuitarMidiPlugin *)instance;
	FretBoard *fretboard = plugin->fretboard;

	switch ((PortIndex)port)
	{
	case FRETBOARD_INPUT:
		plugin->host_input = (const float *)data;
		break;

	case FRETBOARD_MIDIOUTPUT:
		lv2_log_note(&g_logger, "Connecting MIDI OUTPUT PORT\n");
		fretboard->setMidiOutput((LV2_Atom_Sequence *)data);
		break;

	case FRETBOARD_SMOOTHING:
		fretboard->setSmoothing((float *)data);
		break;
	case FRETBOARD_SMOOTHING_OFFSET:
		fretboard->setSmoothingOffset((float *)data);
		break;
	case FRETBOARD_ONSET_THRESHOLD:
		fretboard->setOnsetThreshold((float *)data);
		break;
	case FRETBOARD_OFFSET_THRESHOLD:
		fretboard->setOffsetThreshold((float *)data);
		break;
	case FRETBOARD_ONSET_ENERGY_THRESHOLD:
		fretboard->setOnsetEnergyThreshold((float *)data);
		break;
	case FRETBOARD_OFFSET_ENERGY_THRESHOLD:
		fretboard->setOffsetEnergyThreshold((float *)data);
		break;
#ifdef WITH_AUDIO_OUTPUT
	case FRETBOARD_AUDIO_OUTPUT:
		fretboard->setAudioOutputBuffer((float *)data);
		break;
#endif
	default:
		break;
	}
}

static void
activate(LV2_Handle instance)
{
	GuitarMidiPlugin *plugin = (GuitarMidiPlugin *)instance;
	plugin->buffer_pos = 0;
}

/** Define a macro for converting a gain in dB to a coefficient. */
#define DB_CO(g) ((g) > -90.0f ? powf(10.0f, (g) * 0.05f) : 0.0f)

static void
run(LV2_Handle instance, uint32_t n_samples)
{
	GuitarMidiPlugin *plugin = (GuitarMidiPlugin *)instance;
	FretBoard *fretboard = plugin->fretboard;

	if (n_samples == PROCESS_BLOCK_SIZE) {
		fretboard->setAudioInput(plugin->host_input);
		fretboard->process(n_samples);
		return;
	}

	/* Accumulate smaller blocks into the internal buffer */
	uint32_t remaining = n_samples;
	uint32_t src_offset = 0;

	while (remaining > 0) {
		uint32_t space = PROCESS_BLOCK_SIZE - plugin->buffer_pos;
		uint32_t to_copy = remaining < space ? remaining : space;

		memcpy(plugin->buffer + plugin->buffer_pos,
		       plugin->host_input + src_offset,
		       to_copy * sizeof(float));

		plugin->buffer_pos += to_copy;
		src_offset += to_copy;
		remaining -= to_copy;

		if (plugin->buffer_pos == PROCESS_BLOCK_SIZE) {
			fretboard->setAudioInput(plugin->buffer);
			fretboard->process(PROCESS_BLOCK_SIZE);
			plugin->buffer_pos = 0;
		}
	}
}

static void
deactivate(LV2_Handle instance)
{
	GuitarMidiPlugin *plugin = (GuitarMidiPlugin *)instance;
	plugin->fretboard->finalize();
}

static void
cleanup(LV2_Handle instance)
{
	GuitarMidiPlugin *plugin = (GuitarMidiPlugin *)instance;
	delete plugin->fretboard;
	delete plugin;
}

static const void *
extension_data(const char *uri)
{
	return NULL;
}

static const LV2_Descriptor descriptor = {
	AMP_URI,
	instantiate,
	connect_port,
	activate,
	run,
	deactivate,
	cleanup,
	extension_data};

LV2_SYMBOL_EXPORT
const LV2_Descriptor *
lv2_descriptor(uint32_t index)
{
	switch (index)
	{
	case 0:
		return &descriptor;
	default:
		return NULL;
	}
}
