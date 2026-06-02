<template>
    <div class="max-w-3xl w-full m-auto pt-8 pb-12">
        <AppLoading v-if="processing" message="Processing request..." />
        <AlertsError v-if="errors.message" :message="errors.message" />
        <AlertsSuccess v-if="message" :message="message" @close="message = ''" />
        <div class="w-full">
            <h1 class="text-3xl font-bold mb-3">
                Your apps
            </h1>
            <div class="grid grid-cols-2 gap-4 w-full border-t border-gray-200 pt-6" v-if="owned_apps.length > 0">
                <AppCard :apps="owned_apps" />
            </div>
            <div class="border-t border-gray-200 pt-3" v-else>
                <p>You do not have any owned app</p>
            </div>
            <div class="py-3 mt-3">
                <h1 class="text-lg font-bold mb-1">Create new app</h1>
                <form @submit.prevent="create_app" class="flex flex-col" method="POST">
                    <div class="flex w-full mr-2">
                        <AppInput v-model="title" type="text" placeholder="App title" required />
                        <button v-if="!processing" type="submit"
                            class="max-w-lg px-3 ml-1 bg-main py-2 rounded-lg text-white">Create</button>
                    </div>
                    <AlertsAlertError v-if="errors.title" error="App title is required!" />
                </form>
            </div>

            <div class="py-3 border-t border-gray-200 mt-3" v-if="owned_apps.length > 0">
                <h1 class="text-lg font-bold mb-1">Add member in the app</h1>
                <form @submit.prevent="add_member" class="flex items-center" method="POST">
                    <div class="flex w-full flex-col mr-2">
                        <AppInput v-model="email_address" type="email" class="mr-2" placeholder="Email address" />
                        <AlertsAlertError v-if="errors.email_address" :error="errors.email_address" />
                    </div>
                    <div class="flex w-full flex-col mr-2">
                        <AppSelect v-model="app" placeholder="Select the app">
                            <option :value="app.unique_id" v-for="app in owned_apps" :key="app.unique_id">
                                {{ app.title }}
                            </option>
                        </AppSelect>
                        <AlertsAlertError v-if="errors.app" :error="errors.app" />
                    </div>
                    <button v-if="!processing" type="submit"
                        class="max-w-lg px-3 ml-1 bg-main py-2 rounded-lg text-white">Add</button>
                </form>
            </div>
        </div>
        <div class="w-full">
            <h1 class="text-3xl font-bold mb-3 mt-8">
                Shared apps
            </h1>
            <div class="grid grid-cols-2 gap-4 w-full border-t border-gray-200 pt-6" v-if="shared_apps.length > 0">
                <AppCard :apps="shared_apps" />
            </div>
            <div class="border-t border-gray-200 pt-6" v-else>
                <p>You do not have any shared app</p>
            </div>
        </div>
        <button @click="logout" class="bg-main text-white py-2 px-4 rounded-lg mt-4">Logout</button>
    </div>
</template>
<script lang="js" setup>
definePageMeta({
    middleware: 'auth'
});

const { getToken, removeToken } = useAuthToken();
const title = ref("");
const email_address = ref("");
const app = ref("");
const processing = ref(false);
const message = ref(null);
const errors = ref({
    title: null,
    app: null,
    email_address: null,
    app: null,
    message: null,
    count: 0
})
const owned_apps = ref([]);
const shared_apps = ref([]);


try {
    const { data } = await useFetch('/api/apps/apps', {
        method: "POST",
        body: {
            token: getToken()
        }
    });
    if (data.value.data.length > 0) {
        owned_apps.value = data.value.data.filter(app => app.is_owner === true);
        shared_apps.value = data.value.data.filter(app => app.is_owner === false);
    }
} catch (e) {
    console.log(e)
}

async function create_app() {
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
        const data = await $fetch("/api/apps/create", {
            method: "POST",
            body: {
                token: getToken(),
                title: title.value.trim()
            }
        });
        title.value = "";
        owned_apps.value.push(data.data);
        message.value = data.message;
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to create workspace.';
    } finally {
        processing.value = false;
    }
}

async function add_member() {
    processing.value = true;
    reset_errors();
    if (email_address.value.trim() == "") {
        errors.value.email_address = "Email is required";
        errors.value.count += 1;
    }
    if (!email_address.value.includes("@")) {
        errors.value.email_address = "Wrong email format";
        errors.value.count += 1;
    }
    if (app.value.trim() == "") {
        errors.value.app = "App is required";
        errors.value.count += 1;
    }
    if (errors.value.count > 0) {
        processing.value = false;
        return;
    };
    try {
        const data = await $fetch("/api/apps/add-member", {
            method: "POST",
            body: {
                token: getToken(),
                email_address: email_address.value.trim(),
                app: app.value.trim()
            }
        });
        email_address.value = "";
        app.value = "";
        owned_apps.value.push(data.data);
        message.value = data.message;
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to create workspace.';
    } finally {
        processing.value = false;
    }
}

async function logout() {
    console.log("Logout");
    removeToken();
    await navigateTo('/login');
}

function reset_errors() {
    errors.value = {
        title: null,
        app: null,
        email_address: null,
        message: null,
        count: 0
    };
}
</script>