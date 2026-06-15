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
            <div class="flex flex-col mt-4" v-if="tokens.length > 0">
                <div class="p-3 bg-gray-100 rounded-lg mb-3" v-for="token in tokens" :key="token.unique_id">
                    <strong>{{ token.title }}</strong>
                    <p class="text-[12px]">{{ token.token }}</p>
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
    reset_errors();
    try {
        const data = await $fetch("/api/tokens/create", {
            method: "POST",
            body: {
                token: getToken(),
                app: app.value
            }
        });
        message.value = data.message;
        app.value = "";
    } catch (e) {
        errors.value.message = e.statusMessage || 'Failed to create token.';
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