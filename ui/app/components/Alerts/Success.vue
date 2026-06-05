<template>
    <div class="fixed left-2 bottom-2 bg-dark border border-white/10 rounded-xl max-w-85 w-full z-50">
        <div class="flex items-center justify-between relative py-3 px-4">
            <div class="flex items-center">
                <img src="https://api.iconify.design/ic:baseline-check-circle-outline.svg?color=%232aea40" width="15px">
                <div class="flex flex-col ml-3 border-l border-white/10 pl-3">
                    <h3 class="text-green-500">Action successfull.</h3>
                    <p class="text-[12px] text-white">{{ message }}</p>
                </div>
            </div>
            <button @click="closeMessage" class="ml-4 text-white/50 hover:text-white transition absolute top-2 right-2">
                <img src="https://api.iconify.design/ic:baseline-close.svg?color=%23ffffff" width="18px" alt="Close">
            </button>
        </div>
    </div>
</template>

<script setup>
defineProps({
    message: {
        type: String,
        default: "API called was success."
    }
})

const emit = defineEmits(['close'])

function closeMessage() {
    emit('close')
}
</script>