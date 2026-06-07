<template>
    <div class="flex items-center justify-between relative">
        <AlertsSuccess v-if="message" :message="message" @close="message = ''" />
        <AppLoading v-if="processing" message="Processing request..." />
        <AlertsError v-if="errors.message" :message="errors.message" />
        <div class="flex items-center">
            <img :src="sysFields[field.field_type]" width="18px" />
            <span class="ml-1">{{ field.title }}</span>
        </div>
        <button @click="dropdown = !dropdown">
            <img v-if="!field.is_system"
                src="https://api.iconify.design/material-symbols-light:arrow-drop-down-rounded.svg?color=%233d3846"
                width="25px" :class="{ 'rotate-180': dropdown }">
        </button>
        <div class="absolute top-6 w-55 p-3 -left-3 bg-white rounded-lg shadow z-20" v-show="dropdown">
            <div class="flex flex-col">
                <strong class="mb-2 pb-1 border-b border-gray-200">Action</strong>
                <button class="w-full bg-red-600 flex items-center justify-center p-2 rounded-lg" @click="delete_field">
                    <span class="text-white mr-1">Delete</span>
                    <img src="https://api.iconify.design/solar:trash-bin-minimalistic-bold.svg?color=%23ffffff"
                        width="15px">
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="js">
const sysFields = ref(useFields());
const { getToken } = useAuthToken();

const dropdown = ref(false);
const processing = ref(false);
const message = ref(null);
const errors = ref({});
const props = defineProps({
    field: {
        type: Object
    },
    workspace: String
});
const emit = defineEmits(['field-deleted']);

async function delete_field() {
    processing.value = true;
    try {
        const data = await $fetch("/api/fields/delete", {
            method: "POST",
            body: {
                token: getToken(),
                workspace: props.workspace,
                field_id: props.field.id
            }
        });
        message.value = data.message;
        emit('field-deleted', props.field.id);
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to delete column.';
    } finally {
        processing.value = false;
    }
}
</script>