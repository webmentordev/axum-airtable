export default defineEventHandler(async (event) => {
  const apiUrl = useRuntimeConfig(event).apiUrl;
  const body = await readBody(event);
  try {
    const data = await $fetch(`${apiUrl}/system/records/${body.workspace}?page=${body.page}`, {
      headers: {
            "Authorization": `Bearer ${body.token}`,
            "Content-Type": "application/json",
            "Accept": "application/json"
        },
    });
    if (data.records && data.records.length > 0) {
      data.records = data.records.map(record => ({
        ...record,
        "Created At": new Date(record["Created At"]).toLocaleString() + ' UTC',
        "Updated At": new Date(record["Updated At"]).toLocaleString() + ' UTC'
      }));
    }
    return data;
  } catch (e) {
    throw createError({
      statusCode: e.response?.status || 500,
      statusMessage: e.data.message || 'Records fetch failed'
    });
  }
});