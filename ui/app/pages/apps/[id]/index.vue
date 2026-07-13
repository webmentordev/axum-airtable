<template>
    <div class="w-full h-screen flex flex-col justify-between text-[0.820rem]">
        <AlertsSuccess v-if="message" :message="message" @close="message = ''" />
        <div class="flex flex-col w-full">
            <div class="bg-main px-3 py-1 flex items-center justify-between">
                <div class="flex items-center">
                    <NuxtLink class="text-white text-lg" to="/"><strong>{{ app.title }}</strong></NuxtLink>
                    <ul class="flex items-center ml-4 pl-3 border-l border-white/40">
                        <NuxtLink class="px-3 text-white" to='/'>Apps</NuxtLink>
                        <NuxtLink class="px-3 text-white" :to='`${app.unique_id}`'>Data</NuxtLink>
                        <NuxtLink class="px-3 text-white" :to='`${app.unique_id}/automation`'>Automation
                        </NuxtLink>
                        <NuxtLink class="px-3 text-white" :to='`${app.unique_id}/interfaces`'>Interfaces
                        </NuxtLink>
                    </ul>
                </div>
                <div class="relative flex items-center">
                    <button @click="setting = !setting"><img
                            src="https://api.iconify.design/material-symbols-light:settings.svg?color=%23ffffff"
                            width="20px"></button>
                    <div class="flex flex-col w-40 py-2 px-3 bg-white border border-gray-200 rounded-lg absolute top-8 right-0 z-20"
                        v-show="setting">
                        <strong class="mb-2 pb-1 border-b border-gray-200">Action</strong>
                        <NuxtLink class="mb-1 pb-1" to="/tokens">Tokens</NuxtLink>
                        <form @submit.prevent="delete_app" class="">
                            <Button v-if="!processing" type="submit" text="Delete" class="bg-red-500" />
                        </form>
                    </div>
                </div>
            </div>
            <div class="bg-main-100 px-3 flex items-center">
                <div class="px-4 py-2 rounded-t-sm text-black"
                    :class="{ 'bg-white': active_workspace == workspace.unique_id }" v-for="workspace in workspaces"
                    :key="workspace.unique_id">
                    <button @click="active_workspace = workspace.unique_id">
                        {{ workspace.title }}
                    </button>
                </div>
                <div class="relative ml-2 flex items-center">
                    <button @click="dropdown = !dropdown" class="py-2 px-4">
                        <img src="https://api.iconify.design/material-symbols-light:note-stack-add-rounded.svg?color=%23ffffff"
                            width="20px">
                    </button>
                    <form @submit.prevent="create_workspace"
                        class="w-62.5 p-3 bg-white border border-gray-200 rounded-lg absolute top-8 left-0 z-20"
                        v-show="dropdown">
                        <div class="flex flex-col mb-3">
                            <AppInput v-model="title" type="text" placeholder="Workspace title" />
                            <AlertsAlertError v-if="errors.title" error="Workspace title is required!" />
                        </div>
                        <Button v-if="!processing" type="submit" text="Create" />
                        <AppLoading v-if="processing" message="Processing request..." />
                    </form>
                </div>
            </div>
        </div>
        <div class="gap-6 flex flex-col h-full bg-gray-50 border-b border-slate-100 overflow-x-auto"
            v-show="active_workspace != ''">
            <table class="table-auto records max-w-fit">
                <thead>
                    <tr>
                        <th>ID #</th>
                        <th>Record ID #</th>
                        <th v-for="field in fields" :key="field.id">
                            <AppHeadCell @field-deleted="handleFieldDeleted" :workspace="active_workspace"
                                :field="field" />
                        </th>
                        <th>
                            <AppCreateColumn @field-created="handleFieldCreated" :workspace="active_workspace" />
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <tr class="hover:bg-gray-100" v-if="records.length > 0" v-for="(record, index) in records"
                        :key="record.id">
                        <td><span class="cell inline-block">{{ index + 1 }}</span></td>
                        <td><span class="cell inline-block">{{ record.id }}</span></td>
                        <td v-for="field in fields" :key="field.id">
                            <AppCell :record="record" :field="field" :workspace="active_workspace" />
                        </td>
                        <td>
                            <button class="w-fit flex items-center justify-center rounded-lg bg-red-500 p-1 px-2 m-auto"
                                @click="delete_record(record.id)">
                                <span class="text-white mr-1">Delete</span>
                                <img src="https://api.iconify.design/solar:trash-bin-minimalistic-bold.svg?color=%23ffffff"
                                    width="15px">
                            </button>
                        </td>
                    </tr>
                    <tr v-else>
                        <td colspan="100%">No record found in the table.</td>
                    </tr>
                </tbody>
                <tfoot>
                    <tr>
                        <td>
                            <button @click="create_record" class="flex w-full items-center justify-center">
                                <img src="https://api.iconify.design/material-symbols:add-2-rounded.svg" width="20px" />
                            </button>
                        </td>
                        <td colspan="100%"></td>
                    </tr>
                </tfoot>
            </table>
        </div>
        <div class="flex items-center p-2" v-if="workspaces.length > 0">
            <button @click="delete_workspace"
                class="flex-end py-2 px-4 bg-red-500 text-white rounded-lg text-[10px]">Delete</button>
            <span class="ml-3">Total records: {{ records.length }} / {{ total_records }}</span>
        </div>
    </div>
</template>
<script lang="js" setup>
definePageMeta({
    middleware: 'auth'
});
const { getToken } = useAuthToken();


const records = ref([]);
const fields = ref([]);
const total_records = ref(0);
const current_page = ref(1);
const total_pages = ref(0);

const id = useRoute().params.id;
const app = ref({});
const workspaces = ref([]);
const active_workspace = ref("");
const dropdown = ref(false);
const setting = ref(false);
const processing = ref(false);
const title = ref("");
const message = ref(null);
const errors = ref({
    title: null,
    message: null,
    count: 0
})
try {
    const { data } = await useFetch('/api/apps/app', {
        method: "POST",
        body: {
            token: getToken(),
            id: id
        }
    });
    app.value = data.value.data.app;
    workspaces.value = data.value.data.workspaces;
    if (workspaces.value.length > 0) {
        active_workspace.value = workspaces.value[0].unique_id;
        await fetch_records(active_workspace.value);
    }
} catch (e) {
    console.log(e)
}

watch(active_workspace, async (newWorkspaceId) => {
    if (newWorkspaceId) {
        await fetch_records(newWorkspaceId);
        console.log(newWorkspaceId);
    }
});

async function fetch_records(newWorkspaceID) {
    try {
        const data = await $fetch("/api/records/records", {
            method: "POST",
            body: {
                token: getToken(),
                workspace: newWorkspaceID,
                page: current_page.value
            }
        });
        fields.value = data.fields;
        records.value = data.records;
        total_records.value = data.total_records;
        total_pages.value = data.total_pages;
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to fetch records.';
    }
}


async function create_workspace() {
    processing.value = true;
    reset_errors();
    if (title.value.trim() == "") {
        errors.value.title = "Title is required";
        errors.value.count += 1;
    }
    if (errors.value.count > 0) {
        processing.value = false;
        return;
    };
    try {
        const data = await $fetch("/api/workspace/create", {
            method: "POST",
            body: {
                token: getToken(),
                title: title.value.trim(),
                app_id: app.value.unique_id
            }
        });
        workspaces.value.push(data.data);
        if (workspaces.value.length == 1) {
            active_workspace.value = data.data.unique_id
        }
        title.value = "";
        message.value = data.message;
        dropdown.value = false;
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to create workspace.';
    } finally {
        processing.value = false;
    }
}


async function delete_workspace() {
    processing.value = true;
    reset_errors();
    try {
        const data = await $fetch("/api/workspace/delete", {
            method: "POST",
            body: {
                token: getToken(),
                app_id: app.value.unique_id,
                workspace_id: active_workspace.value
            }
        });
        workspaces.value = workspaces.value.filter(
            w => w.unique_id !== active_workspace.value
        );
        if (workspaces.value.length > 0) {
            active_workspace.value = workspaces.value[workspaces.value.length - 1].unique_id;
        } else {
            active_workspace.value = null;
        }
        message.value = data.message;
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to delete workspace.';
    } finally {
        processing.value = false;
    }
}

async function delete_app() {
    processing.value = true;
    try {
        const data = await $fetch("/api/apps/delete", {
            method: "POST",
            body: {
                token: getToken(),
                app_id: app.value.unique_id,
            }
        });
        navigateTo("/");
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to delete app.';
    } finally {
        processing.value = false;
    }
}

async function create_record() {
    processing.value = true;
    try {
        const data = await $fetch("/api/records/create", {
            method: "POST",
            body: {
                token: getToken(),
                workspace: active_workspace.value,
            }
        });
        message.value = data.message;
        records.value.push(data.record);
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to create record.';
    } finally {
        processing.value = false;
    }
}

async function delete_record(recordID) {
    processing.value = true;
    try {
        const data = await $fetch("/api/records/delete", {
            method: "POST",
            body: {
                token: getToken(),
                workspace: active_workspace.value,
                record: recordID
            }
        });
        message.value = data.message;
        records.value = records.value.filter(r => r.id !== recordID);
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to delete record.';
    } finally {
        processing.value = false;
    }
}

const handleFieldDeleted = (fieldId) => {
    const fieldToDelete = fields.value.find(f => f.id === fieldId);
    fields.value = fields.value.filter(f => f.id !== fieldId);
    if (fieldToDelete) {
        records.value = records.value.map(record => {
            const updatedRecord = { ...record };
            delete updatedRecord[fieldToDelete.title];
            return updatedRecord;
        });
    }
};

const handleFieldCreated = (field) => {
    fields.value.push(field);
};

function reset_errors() {
    errors.value = {
        title: null,
        message: null,
        count: 0
    };
}

</script>