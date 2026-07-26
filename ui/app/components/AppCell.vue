<template>
    <div v-if="!field.is_system">
        <AlertsSaved v-if="saved" />
        <input type="text" v-model="field_value" @keydown="on_input" @blur="update_record()"
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