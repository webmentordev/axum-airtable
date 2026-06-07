<template>
    <div class="relative">
        <AlertsSuccess v-if="message" :message="message" @close="message = ''" />
        <AppLoading v-if="processing" message="Processing request..." />
        <AlertsError v-if="errors.message" :message="errors.message" />
        <button @click="dropdown = !dropdown" class="flex items-center justify-center w-full">
            <img src="https://api.iconify.design/material-symbols:add-2-rounded.svg" width="20px" />
            <span class="ml-1">Add column</span>
        </button>
        <div class="absolute top-6 w-65 p-3 -right-3 bg-white rounded-lg shadow z-20" v-show="dropdown">
            <div class="flex flex-col">
                <strong class="mb-2 pb-1 border-b border-gray-200">Action</strong>
                <div class="flex flex-col mb-3">
                    <AppInput v-model="title" type="text" placeholder="Column name" required />
                    <AlertsAlertError v-if="errors.title" :error="errors.title" />
                </div>
                <div class="flex flex-col mb-3">
                    <AppSelect v-model="field_type" placeholder="Select the column type" required>
                        <option :value="f_type" v-for="(f_type, index) in filed_types" :key="index">
                            {{ f_type }}
                        </option>
                    </AppSelect>
                    <AlertsAlertError v-if="errors.field_type" :error="errors.field_type" />
                </div>
                <Button @click="create_column" text="Create" />
            </div>
        </div>
    </div>
</template>

<script setup lang="js">
const { getToken } = useAuthToken();

const dropdown = ref(false);
const processing = ref(false);
const message = ref(null);
const errors = ref({
    count: 0
});

const title = ref("");
const field_type = ref("");
const filed_types = ref([
    "Text",
    "Email",
    "Phone",
    "Date",
    "Attachments",
    "Multi_select",
    "Checkbox",
    "Json",
    "Currency"
]);

const emit = defineEmits(['field-created']);
const props = defineProps({
    workspace: String
});

async function create_column() {
    processing.value = true;
    reset_errors();
    if (title.value.trim() == "") {
        errors.value.title = "Title is required.";
        errors.value.count += 1;
    }
    if (field_type.value.trim() == "") {
        errors.value.field_type = "Column type is required.";
        errors.value.count += 1;
    }
    if (errors.value.count > 0) {
        processing.value = false;
        return;
    }
    try {
        const data = await $fetch("/api/fields/create", {
            method: "POST",
            body: {
                token: getToken(),
                workspace: props.workspace,
                title: title.value,
                field_type: field_type.value
            }
        });
        message.value = data.message;
        emit('field-created', data.data);
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to create a column.';
    } finally {
        processing.value = false;
    }
}

function reset_errors() {
    errors.value = {
        count: 0
    };
}
</script>