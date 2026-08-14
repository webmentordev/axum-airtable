<template>
    <div v-if="!field.is_system">
        <AlertsSaved v-if="saved" />
        <input v-if="fieldTypeConfig.input_type === 'text'" type="text" v-model="field_value" @keydown="on_input"
            @blur="update_record()" class="recinput w-full h-full" required>

        <input v-else-if="fieldTypeConfig.input_type === 'email'" type="email" v-model="field_value" @keydown="on_input"
            @blur="update_record()" class="recinput w-full h-full" required>

        <input v-else-if="fieldTypeConfig.input_type === 'phone'" type="tel" v-model="field_value" @keydown="on_input"
            @blur="update_record()" class="recinput w-full h-full" placeholder="+1 (555) 000-0000">

        <input v-else-if="fieldTypeConfig.input_type === 'number'" type="number" v-model.number="field_value"
            @keydown="on_input" @blur="update_record()" class="recinput w-full h-full"
            :step="fieldTypeConfig.step || 1">

        <div v-else-if="fieldTypeConfig.input_type === 'currency'" class="flex items-center">
            <span class="pr-2">$</span>
            <input type="number" v-model.number="field_value" @keydown="on_input" @blur="update_record()"
                class="recinput w-full h-full" step="0.01" placeholder="0.00">
        </div>

        <input v-else-if="fieldTypeConfig.input_type === 'checkbox'" type="checkbox" v-model="field_value"
            @change="update_record()" class="w-4 h-4 cursor-pointer">

        <input v-else-if="fieldTypeConfig.input_type === 'date'" type="date" v-model="field_value"
            @change="update_record()" class="recinput w-full h-full">

        <input v-else-if="fieldTypeConfig.input_type === 'date_readonly'" type="date" v-model="field_value" disabled
            class="recinput w-full h-full bg-gray-100">

        <select v-else-if="fieldTypeConfig.input_type === 'multi_select'" v-model="field_value" multiple
            @change="update_record()" class="recinput w-full h-full">
            <option value="">Select options...</option>
        </select>

        <input v-else-if="fieldTypeConfig.input_type === 'attachments'" type="file" multiple
            @change="handle_file_upload" class="recinput w-full h-full">

        <input v-else type="text" v-model="field_value" @keydown="on_input" @blur="update_record()"
            class="recinput w-full h-full" required>
    </div>
    <span v-else class="cursor-pointer w-full h-full cell">{{ field_value }}</span>
</template>

<script setup lang="js">
const { getToken } = useAuthToken();

const props = defineProps({
    record: Object,
    field: Object,
    workspace: String
});

const fieldTypeMap = ref({
    'text': { input_type: 'text' },
    'email': { input_type: 'email' },
    'phone': { input_type: 'phone' },
    'number': { input_type: 'number', step: 1 },
    'currency': { input_type: 'currency', step: 0.01 },
    'checkbox': { input_type: 'checkbox' },
    'date': { input_type: 'date' },
    'created_at': { input_type: 'date_readonly' },
    'updated_at': { input_type: 'date_readonly' },
    'multi_select': { input_type: 'multi_select' },
    'attachments': { input_type: 'attachments' },
});

const fieldTypeConfig = computed(() => {
    return fieldTypeMap.value[props.field.field_type] || { input_type: 'text' };
});

const unsaved = ref(false);
const saved = ref(false);
const field_value = ref(props.record[props.field.title] || "");
const processing = ref(false);
const errors = ref({
    count: 0
});

function on_input() {
    unsaved.value = field_value.value !== props.record[props.field.title];
}

async function update_record() {
    if (!unsaved.value) return;
    processing.value = true;

    try {
        const data = await $fetch("/api/records/update", {
            method: "POST",
            body: {
                token: getToken(),
                workspace: props.workspace,
                record: props.record.id,
                field: props.field.id ? props.field.id : props.field.unique_id,
                field_value: field_value.value
            }
        });
        saved.value = true;
        setTimeout(() => {
            saved.value = false;
        }, 2000);
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to update record.';
    } finally {
        processing.value = false;
    }
}

</script>