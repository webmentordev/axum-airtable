<template>
    <div class="max-w-3xl w-full m-auto pt-8 pb-12">
        <AppLoading v-if="processing" message="Processing request..." />
        <AlertsError v-if="errors.message" :message="errors.message" />
        <AlertsSuccess v-if="message" :message="message" @close="message = ''" />
        <h2 class="text-2xl">Your API Tokens</h2>
        <div class="py-3 border-t border-gray-200 mt-3" v-if="apps.length > 0">
            <h1 class="text-lg font-bold mb-1">Create API token</h1>
            <form @submit.prevent="create_token" class="flex items-center" method="POST">
                <div class="flex w-full flex-col mr-2">
                    <AppSelect v-model="app" placeholder="Select the app">
                        <option :value="app.unique_id" v-for="app in apps" :key="app.unique_id">
                            {{ app.title }}
                        </option>
                    </AppSelect>
                    <AlertsAlertError v-if="errors.app" :error="errors.app" />
                </div>
                <button v-if="!processing" type="submit"
                    class="max-w-lg px-3 ml-1 bg-main py-2 rounded-lg text-white">Create</button>
            </form>

            <div v-if="created.token" class="p-4 bg-gray-100 rounded-lg border border-gray-400 mt-4">
                <h2 class="text-lg mb-1">API token created!</h2>
                <div class="flex items-center">
                    <p>{{ created.token }}</p>
                    <button @click="copyToClipboard"
                        class="bg-white p-1 rounded-lg ml-3 border border-gray-500 hover:bg-gray-50 transition">
                        <img src="https://api.iconify.design/mynaui:copy.svg" width="15px">
                    </button>
                    <span v-if="copied" class="text-green-600 ml-2 transition">Copied!</span>
                </div>
                <p class="text-red-500">This token will not be visible again.</p>
            </div>

            <div class="flex flex-col mt-4" v-if="tokens.length > 0">
                <div class="p-3 bg-gray-100 rounded-lg mb-3" v-for="token in tokens" :key="token.unique_id">
                    <div class="flex items-start justify-between">
                        <div>
                            <div class="flex items-center mb-2">
                                <strong class="mr-1">Workspace:</strong>
                                <span>{{ token.title }}</span>
                            </div>
                            <p>{{ token.token }}</p>
                            <p class="text-[12px]"><strong>Created:</strong> {{ new
                                Date(token.created_at).toLocaleString()
                                +
                                ' UTC' }}</p>
                        </div>
                        <button @click="delete_token(token.unique_id, token.app_id)"
                            class="p-1 bg-white border border-gray-200 rounded-lg">
                            <img src="https://api.iconify.design/material-symbols:delete-outline.svg?color=%23e01b24"
                                width="20px">
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
<script lang="js" setup>
definePageMeta({
    middleware: 'auth'
});

const { getToken } = useAuthToken();
const tokens = ref([]);
const apps = ref([]);
const app = ref("");
const processing = ref(false);
const message = ref(null);
const created = ref({});
const copied = ref(false)
const errors = ref({
    count: 0
})

try {
    const { data } = await useAsyncData('apps-and-tokens', async () => {
        const [apps, tokens] = await Promise.all([
            $fetch('/api/apps/apps', {
                method: "POST",
                body: { token: getToken() }
            }),
            $fetch('/api/tokens/tokens', {
                method: "POST",
                body: { token: getToken(), app: app.value }
            })
        ])
        return { apps, tokens }
    })
    if (data.value.apps.data.length > 0) {
        apps.value = data.value.apps.data;
    }
    if (data.value.tokens) {
        tokens.value = data.value.tokens.data;
    }
} catch (e) {
    errors.value.message = e.statusMessage || 'Failed to fetch apps and tokens.'
}

async function create_token() {
    processing.value = true;
    resetItems();
    try {
        const data = await $fetch("/api/tokens/create", {
            method: "POST",
            body: {
                token: getToken(),
                app: app.value
            }
        });
        message.value = data.message;
        created.value = data.data;
        app.value = "";
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to create token.';
    } finally {
        processing.value = false;
    }
}

async function delete_token(tokenId, appId) {
    processing.value = true;
    try {
        const data = await $fetch("/api/tokens/delete", {
            method: "POST",
            body: {
                token: getToken(),
                app: appId,
                token_id: tokenId
            }
        });
        message.value = data.message;
        tokens.value = tokens.value.filter(f => f.unique_id !== tokenId);
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to delete the token.';
    } finally {
        processing.value = false;
    }
}

const copyToClipboard = async () => {
    try {
        await navigator.clipboard.writeText(created.value.token)
        copied.value = true
        setTimeout(() => {
            copied.value = false
        }, 2000)
    } catch (err) {
        console.error('Failed to copy:', err)
    }
}

function resetItems() {
    created.value = {};
    errors.value = {
        count: 0
    };
}
</script>